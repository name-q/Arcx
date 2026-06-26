//! 日志系统
//!
//! 框架级日志，提供：
//! - 分级日志（DEBUG/INFO/WARN/ERROR）
//! - 日志分文件（app.log / error.log）
//! - 自动带 trace_id（请求追踪）
//! - 配置化（级别、路径、格式）
//!
//! 配置方式：
//! ```toml
//! [logger]
//! level = "info"         # 全局日志级别
//! dir = "logs"           # 日志目录
//! enable_console = true  # 是否输出到控制台
//! enable_file = true     # 是否输出到文件
//! ```

pub mod trace_id;

use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};
use std::path::Path;
use std::fs;

/// 日志配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoggerConfig {
    /// 日志级别: trace, debug, info, warn, error
    #[serde(default = "default_level")]
    pub level: String,
    /// 日志目录
    #[serde(default = "default_dir")]
    pub dir: String,
    /// 是否输出到控制台
    #[serde(default = "default_true")]
    pub enable_console: bool,
    /// 是否输出到文件
    #[serde(default = "default_false")]
    pub enable_file: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: default_level(),
            dir: default_dir(),
            enable_console: true,
            enable_file: false,
        }
    }
}

fn default_level() -> String { "info".to_string() }
fn default_dir() -> String { "logs".to_string() }
fn default_true() -> bool { true }
fn default_false() -> bool { false }

/// 初始化日志系统
/// 必须在应用启动最前面调用
pub fn init(config: &LoggerConfig) {
    // 确保日志目录存在
    if config.enable_file {
        let log_dir = Path::new(&config.dir);
        if !log_dir.exists() {
            fs::create_dir_all(log_dir).expect("Failed to create log directory");
        }
    }

    // 构建 EnvFilter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let registry = tracing_subscriber::registry().with(filter);

    // 控制台输出层
    let console_layer = if config.enable_console {
        Some(
            fmt::layer()
                .with_target(false)
                .with_timer(fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()))
                .with_ansi(true)
        )
    } else {
        None
    };

    // 文件输出层 — app.log（全量日志）
    let file_layer = if config.enable_file {
        let app_log = tracing_appender::rolling::daily(&config.dir, "app.log");
        Some(
            fmt::layer()
                .with_target(false)
                .with_timer(fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()))
                .with_ansi(false)
                .with_writer(app_log)
        )
    } else {
        None
    };

    // 错误日志层 — error.log（仅 WARN+ERROR）
    let error_layer = if config.enable_file {
        let error_log = tracing_appender::rolling::daily(&config.dir, "error.log");
        let error_filter = EnvFilter::new("warn");
        Some(
            fmt::layer()
                .with_target(false)
                .with_timer(fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()))
                .with_ansi(false)
                .with_writer(error_log)
                .with_filter(error_filter)
        )
    } else {
        None
    };

    registry
        .with(console_layer)
        .with(file_layer)
        .with(error_layer)
        .init();
}
