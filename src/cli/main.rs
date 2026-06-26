//! Arcx CLI 脚手架工具
//!
//! 命令：
//! - arcx new <project>              创建新项目
//! - arcx generate controller <name> 生成 controller 模板
//! - arcx generate service <name>    生成 service 模板
//! - arcx generate job <name>        生成定时任务模板
//! - arcx dev                        启动开发服务器（带热重载提示）

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "arcx", version, about = "Arcx framework CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建新项目
    New {
        /// 项目名称
        name: String,
    },
    /// 生成代码模板
    #[command(alias = "g")]
    Generate {
        #[command(subcommand)]
        what: GenerateTarget,
    },
    /// 启动开发服务
    Dev,
}

#[derive(Subcommand)]
enum GenerateTarget {
    /// 生成 controller
    #[command(alias = "c")]
    Controller { name: String },
    /// 生成 service
    #[command(alias = "s")]
    Service { name: String },
    /// 生成定时任务
    #[command(alias = "j")]
    Job { name: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => cmd_new(&name),
        Commands::Generate { what } => match what {
            GenerateTarget::Controller { name } => cmd_generate_controller(&name),
            GenerateTarget::Service { name } => cmd_generate_service(&name),
            GenerateTarget::Job { name } => cmd_generate_job(&name),
        },
        Commands::Dev => cmd_dev(),
    }
}

/// arcx new <project>
fn cmd_new(name: &str) {
    let project_dir = Path::new(name);
    if project_dir.exists() {
        eprintln!("✗ Directory '{}' already exists", name);
        std::process::exit(1);
    }

    println!("Creating new Arcx project: {}", name);

    // 创建目录结构
    let dirs = [
        "src/controller",
        "src/service",
        "src/middleware",
        "src/plugin",
        "src/schedule",
        "src/guard",
        "config",
    ];
    for dir in &dirs {
        fs::create_dir_all(project_dir.join(dir)).unwrap();
    }

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
arcx = {{ path = "../Arcx" }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tracing = "0.1"
"#
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // config/config.default.toml
    let default_config = format!(
        r#"[app]
name = "{name}"
version = "0.1.0"
env = "dev"

[server]
host = "127.0.0.1"
port = 3000

[middleware]
cors = true
logger = true

[schedule]
enable = false
"#
    );
    fs::write(
        project_dir.join("config/config.default.toml"),
        default_config,
    )
    .unwrap();

    // src/main.rs 骨架
    let main_rs = r#"use arcx::prelude::*;

#[tokio::main]
async fn main() {
    arcx::app::run().await;
}
"#;
    fs::write(project_dir.join("src/main.rs"), main_rs).unwrap();

    // src/controller/mod.rs
    fs::write(project_dir.join("src/controller/mod.rs"), "// Controllers\n").unwrap();

    // .gitignore
    let gitignore = "target/\n*.db\n.idea/\n";
    fs::write(project_dir.join(".gitignore"), gitignore).unwrap();

    println!("✓ Project created at ./{}", name);
    println!();
    println!("  cd {}", name);
    println!("  cargo run");
    println!();
}

/// arcx generate controller <name>
fn cmd_generate_controller(name: &str) {
    let path = format!("src/controller/{}.rs", name);
    if Path::new(&path).exists() {
        eprintln!("✗ Controller '{}' already exists at {}", name, path);
        std::process::exit(1);
    }

    let content = format!(
        r#"//! {name} Controller
//! 路由前缀: /api/{name}

use axum::{{routing::get, Json, Router}};

use crate::context::AppState;
use crate::error::{{AppResult, success}};

/// 公开路由
pub fn routes() -> Router<AppState> {{
    Router::new()
        .route("/", get(index))
}}

/// GET /api/{name}
async fn index() -> AppResult<Json<serde_json::Value>> {{
    Ok(success(serde_json::json!({{
        "message": "{name} controller works"
    }})))
}}
"#
    );

    fs::write(&path, content).unwrap();
    println!("✓ Created: {}", path);
    println!();
    println!("  Don't forget to register in src/controller/mod.rs:");
    println!("    pub mod {};", name);
    println!();
    println!("  And in src/router/mod.rs register_controllers! macro:");
    println!("    register_controllers!({}, ...);", name);
}

/// arcx generate service <name>
fn cmd_generate_service(name: &str) {
    let path = format!("src/service/{}.rs", name);
    if Path::new(&path).exists() {
        eprintln!("✗ Service '{}' already exists at {}", name, path);
        std::process::exit(1);
    }

    let struct_name = to_pascal_case(name);
    let content = format!(
        r#"//! {struct_name} Service

pub struct {struct_name}Service;

impl {struct_name}Service {{
    /// 示例方法
    pub async fn find_all() -> Vec<String> {{
        vec![]
    }}
}}
"#
    );

    fs::write(&path, content).unwrap();
    println!("✓ Created: {}", path);
    println!();
    println!("  Register in src/service/mod.rs:");
    println!("    pub mod {};", name);
}

/// arcx generate job <name>
fn cmd_generate_job(name: &str) {
    let path = format!("src/service/{}_job.rs", name);
    if Path::new(&path).exists() {
        eprintln!("✗ Job '{}' already exists at {}", name, path);
        std::process::exit(1);
    }

    let struct_name = to_pascal_case(name);
    let content = format!(
        r#"//! {struct_name} 定时任务

use crate::schedule::{{JobContext, ScheduleJob}};

pub struct {struct_name}Job;

#[async_trait::async_trait]
impl ScheduleJob for {struct_name}Job {{
    fn name(&self) -> &str {{
        "{name}"
    }}

    fn cron(&self) -> &str {{
        "0 */5 * * * *" // 每5分钟
    }}

    async fn run(&self, ctx: &JobContext) {{
        tracing::info!("[{struct_name}Job] running");
        // TODO: 实现任务逻辑
    }}
}}
"#
    );

    fs::write(&path, content).unwrap();
    println!("✓ Created: {}", path);
    println!();
    println!("  Register in src/service/mod.rs:");
    println!("    pub mod {}_job;", name);
    println!();
    println!("  Register in src/main.rs register_schedule_jobs():");
    println!("    sm.register(service::{}_job::{}Job);", name, struct_name);
}

/// arcx dev
fn cmd_dev() {
    println!("Starting Arcx dev server...");
    println!();
    println!("  Tip: Use `cargo watch -x run` for hot reload");
    println!("  Install: cargo install cargo-watch");
    println!();

    // 直接启动 arcx-server
    let status = std::process::Command::new("cargo")
        .args(["run", "--bin", "arcx-server"])
        .status()
        .expect("Failed to start server");

    std::process::exit(status.code().unwrap_or(1));
}

/// 下划线转 PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}
