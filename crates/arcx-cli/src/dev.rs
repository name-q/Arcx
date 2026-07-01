//! arcx dev — 开发服务器，内置热重载
//!
//! 架构设计：
//! - CLI 进程（主进程）：持有文件监听器 + 管理子进程生命周期
//! - 业务子进程：编译后的 binary，独立进程
//!
//! 热更新策略：
//! 1. 代码变更（.rs）  → 增量编译 → 编译成功后 graceful restart 子进程
//! 2. 配置变更（.toml）→ 发送 SIGHUP 通知子进程重载配置（框架 ConfigWatcher 接管）
//! 3. Cargo.toml 变更  → 全量重编译 + restart
//!
//! 关键优化：
//! - 先编译成功，再杀旧进程 → 最小化服务中断窗口（约 50ms）
//! - SIGTERM 优雅退出，等子进程处理完当前请求
//! - 增量编译，只改一个文件不会全量重编

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use std::{fs, thread};

/// 变更类型
#[derive(Debug, Clone, PartialEq)]
enum ChangeKind {
    /// .rs 文件变更，需要重编译
    Code,
    /// config/*.toml 变更，只需通知子进程
    Config,
    /// Cargo.toml 变更，可能需要下载依赖
    Dependency,
}

pub fn run(cli_port: u16) {
    // 读取配置文件端口
    let port = if cli_port != 8765 {
        cli_port
    } else {
        read_config_port().unwrap_or(8765)
    };

    println!("⚡ Arcx dev server");
    println!("  Port: {}", port);
    println!("  Hot reload: enabled");
    println!("  Watching: src/, config/");
    println!();

    // Ctrl+C 信号处理
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl+C handler");

    // 首次编译并启动
    let mut child: Option<Child> = match build_project() {
        Ok(_) => Some(spawn_child(port)),
        Err(e) => {
            eprintln!("  ✗ Initial build failed: {}", e);
            eprintln!("  Waiting for file changes to retry...");
            None
        }
    };

    // 文件监听器（500ms 防抖）
    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(500), tx)
        .expect("Failed to create file watcher");

    // 监听目录
    for dir in &["src", "config", "Cargo.toml"] {
        let path = Path::new(dir);
        if path.exists() {
            let mode = if path.is_dir() {
                notify::RecursiveMode::Recursive
            } else {
                notify::RecursiveMode::NonRecursive
            };
            debouncer.watcher().watch(path, mode).ok();
        }
    }

    // 主事件循环
    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(events)) => {
                // 判断变更类型
                let change = classify_events(&events);
                if let Some(change_kind) = change {
                    handle_change(change_kind, port, &mut child);
                }
            }
            Ok(Err(errs)) => {
                eprintln!("  Watch error: {:?}", errs);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // 检查子进程是否意外挂了
                check_child_health(&mut child);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // 优雅退出
    println!("\n  Shutting down...");
    if let Some(ref mut c) = child {
        graceful_kill(c);
    }
    println!("  Stopped.");
}

/// 分析事件，确定变更类型
fn classify_events(events: &[notify_debouncer_mini::DebouncedEvent]) -> Option<ChangeKind> {
    let mut has_code = false;
    let mut has_config = false;
    let mut has_dep = false;

    for event in events {
        if !matches!(event.kind, DebouncedEventKind::Any) {
            continue;
        }

        let path = &event.path;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // 忽略非相关文件
        if ext != "rs" && ext != "toml" {
            continue;
        }

        // Cargo.toml 变更 → 依赖变更
        if file_name == "Cargo.toml" {
            has_dep = true;
            continue;
        }

        // config/ 下的 .toml → 配置变更
        let path_str = path.to_string_lossy();
        if path_str.contains("config") && ext == "toml" {
            has_config = true;
            continue;
        }

        // .rs 文件 → 代码变更
        if ext == "rs" {
            has_code = true;
        }
    }

    // 优先级：dep > code > config
    if has_dep || has_code {
        if has_dep {
            Some(ChangeKind::Dependency)
        } else {
            Some(ChangeKind::Code)
        }
    } else if has_config {
        Some(ChangeKind::Config)
    } else {
        None
    }
}

/// 处理变更
fn handle_change(kind: ChangeKind, port: u16, child: &mut Option<Child>) {
    match kind {
        ChangeKind::Config => {
            println!("\n  ⟳ Config changed, notifying process...");
            // 给子进程发 SIGHUP，让 ConfigWatcher 处理
            if let Some(ref c) = child {
                send_signal(c.id(), Signal::Hup);
                println!("  ✓ Config reload signal sent");
            } else {
                println!("  (no running process, config will apply on next start)");
            }
        }
        ChangeKind::Code | ChangeKind::Dependency => {
            let label = if kind == ChangeKind::Dependency {
                "Cargo.toml changed, rebuilding (may fetch deps)..."
            } else {
                "Code changed, rebuilding..."
            };
            println!("\n  ⟳ {}", label);

            let start = Instant::now();

            // 先编译，成功后再杀旧进程（最小化中断窗口）
            match build_project() {
                Ok(_) => {
                    let elapsed = start.elapsed();
                    println!(
                        "  ✓ Build success ({:.1}s), restarting...",
                        elapsed.as_secs_f64()
                    );

                    // 编译成功 → 杀旧进程 → 启动新进程
                    if let Some(ref mut c) = child {
                        graceful_kill(c);
                    }
                    *child = Some(spawn_child(port));
                }
                Err(e) => {
                    eprintln!("  ✗ Build failed: {}", e);
                    eprintln!("  Fix errors and save to retry...");
                    // 旧进程继续跑，不杀
                }
            }
        }
    }
}

/// 编译项目（增量编译）
fn build_project() -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["build", "--color=always"])
        .status()
        .map_err(|e| format!("Failed to run cargo: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("compilation error".to_string())
    }
}

/// 启动子进程
fn spawn_child(port: u16) -> Child {
    let bin_name = get_bin_name().unwrap_or_else(|| "app".to_string());
    let bin_path = format!("target/debug/{}", bin_name);

    Command::new(&bin_path)
        .env("ARCX_PORT", port.to_string())
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to start {}: {}", bin_path, e))
}

/// 优雅杀死子进程：SIGTERM → 等 3s → SIGKILL
fn graceful_kill(child: &mut Child) {
    let pid = child.id();

    // 先发 SIGTERM
    send_signal(pid, Signal::Term);

    // 等待最多 3 秒
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return, // 已退出
            Ok(None) => {
                if Instant::now() >= deadline {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return,
        }
    }

    // 超时，强制 SIGKILL
    eprintln!("  (process didn't exit in 3s, force killing)");
    child.kill().ok();
    child.wait().ok();
}

/// 检查子进程健康
fn check_child_health(child: &mut Option<Child>) {
    if let Some(ref mut c) = child {
        match c.try_wait() {
            Ok(Some(status)) if !status.success() => {
                eprintln!("  ✗ Process crashed ({}), waiting for changes...", status);
                *child = None;
            }
            _ => {}
        }
    }
}

/// 跨平台信号发送
enum Signal {
    Term,
    Hup,
}

fn send_signal(pid: u32, signal: Signal) {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal as NixSignal};
        use nix::unistd::Pid;
        let sig = match signal {
            Signal::Term => NixSignal::SIGTERM,
            Signal::Hup => NixSignal::SIGHUP,
        };
        kill(Pid::from_raw(pid as i32), sig).ok();
    }

    #[cfg(windows)]
    {
        // Windows 没有信号概念，直接 kill
        let _ = signal; // suppress unused warning
        unsafe {
            let handle = winapi::um::processthreadsapi::OpenProcess(
                winapi::um::winnt::PROCESS_TERMINATE,
                0,
                pid,
            );
            if !handle.is_null() {
                winapi::um::processthreadsapi::TerminateProcess(handle, 1);
                winapi::um::handleapi::CloseHandle(handle);
            }
        }
    }
}

fn get_bin_name() -> Option<String> {
    let content = fs::read_to_string("Cargo.toml").ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name") {
            return trimmed
                .split('=')
                .nth(1)
                .map(|v| v.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn read_config_port() -> Option<u16> {
    let config_path = Path::new("config/config.default.toml");
    let content = fs::read_to_string(config_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("port") {
            if let Some(val) = trimmed.split('=').nth(1) {
                return val.trim().parse().ok();
            }
        }
    }
    None
}
