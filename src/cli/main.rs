//! Arcx CLI 脚手架工具
//!
//! 命令：
//! - arcx new <project>              创建新项目
//! - arcx generate controller <name> 生成 controller 模板
//! - arcx generate service <name>    生成 service 模板
//! - arcx generate model <name>      生成 model 模板
//! - arcx generate job <name>        生成定时任务模板
//! - arcx dev                        启动开发服务器（支持热重载）
//! - arcx info                       显示项目信息

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;

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
        /// 项目名称或路径
        name: String,
    },
    /// 生成代码模板
    #[command(alias = "g")]
    Generate {
        #[command(subcommand)]
        what: GenerateTarget,
    },
    /// 启动开发服务（支持热重载）
    Dev {
        /// 指定端口
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// 显示项目信息
    Info,
}

#[derive(Subcommand)]
enum GenerateTarget {
    /// 生成 controller
    #[command(alias = "c")]
    Controller { name: String },
    /// 生成 service
    #[command(alias = "s")]
    Service { name: String },
    /// 生成 model
    #[command(alias = "m")]
    Model { name: String },
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
            GenerateTarget::Model { name } => cmd_generate_model(&name),
            GenerateTarget::Job { name } => cmd_generate_job(&name),
        },
        Commands::Dev { port } => cmd_dev(port),
        Commands::Info => cmd_info(),
    }
}

// ─────────────────────────────────────────
// arcx new <project>
// ─────────────────────────────────────────

fn cmd_new(name: &str) {
    let project_path = Path::new(name);
    // 提取项目名（最后一段路径）
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);

    if project_path.exists() {
        eprintln!("✗ Directory '{}' already exists", name);
        std::process::exit(1);
    }

    println!("⚡ Creating new Arcx project: {}", project_name);
    println!();

    // 创建目录结构
    let dirs = [
        "src/controller",
        "src/service",
        "src/model",
        "src/middleware",
        "src/plugin",
        "src/schedule",
        "src/guard",
        "config",
    ];
    for dir in &dirs {
        fs::create_dir_all(project_path.join(dir)).unwrap();
    }

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
arcx = "0.1"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tracing = "0.1"
validator = {{ version = "0.18", features = ["derive"] }}
async-trait = "0.1"
"#
    );
    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    // config/config.default.toml
    let default_config = format!(
        r#"# Arcx 默认配置

[app]
name = "{project_name}"
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

# 数据库插件（按需启用）
# [plugin.database]
# enable = true
# url = "sqlite:./data.db?mode=rwc"
# max_connections = 10

# JWT 鉴权插件（按需启用）
# [plugin.jwt]
# enable = true
# secret = "your-secret-key"
# expire_hours = 24
"#
    );
    fs::write(
        project_path.join("config/config.default.toml"),
        default_config,
    )
    .unwrap();

    // config/config.prod.toml
    let prod_config = r#"# 生产环境配置（覆盖 default）

[app]
env = "prod"

[server]
host = "0.0.0.0"
port = 8080

[middleware]
logger = true
cors = false
"#;
    fs::write(project_path.join("config/config.prod.toml"), prod_config).unwrap();

    // src/main.rs
    let main_rs = format!(
        r#"//! {project_name} - Powered by Arcx Framework

mod controller;
mod service;

#[tokio::main]
async fn main() {{
    // 初始化日志
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .init();

    tracing::info!("{project_name} starting...");

    // TODO: 初始化框架并启动服务
    // 详细用法参考 Arcx 文档
    tracing::info!("{project_name} ready");
}}
"#
    );
    fs::write(project_path.join("src/main.rs"), main_rs).unwrap();

    // src/controller/mod.rs
    let controller_mod = r#"//! Controller 层
//! 约定：每个文件对应一个资源，文件名即路由前缀
//! 例: user.rs → /api/user

pub mod home;
"#;
    fs::write(project_path.join("src/controller/mod.rs"), controller_mod).unwrap();

    // src/controller/home.rs (示例 controller)
    let home_controller = format!(
        r#"//! Home Controller
//! 路由前缀: /api/home

use axum::{{routing::get, Json, Router}};
use serde_json::{{json, Value}};

pub fn routes() -> Router {{
    Router::new()
        .route("/", get(index))
}}

/// GET /api/home
async fn index() -> Json<Value> {{
    Json(json!({{
        "name": "{project_name}",
        "message": "Welcome to Arcx!"
    }}))
}}
"#
    );
    fs::write(project_path.join("src/controller/home.rs"), home_controller).unwrap();

    // src/service/mod.rs
    fs::write(
        project_path.join("src/service/mod.rs"),
        "//! Service 层\n//! 约定：每个 service 封装一组业务逻辑\n",
    )
    .unwrap();

    // src/model/mod.rs
    fs::write(
        project_path.join("src/model/mod.rs"),
        "//! Model 层\n//! 约定：每个 model 对应一张表（SeaORM Entity）\n",
    )
    .unwrap();

    // .gitignore
    let gitignore = r#"target/
*.db
.idea/
.env
.DS_Store
"#;
    fs::write(project_path.join(".gitignore"), gitignore).unwrap();

    // README.md
    let readme = format!(
        r#"# {project_name}

Built with [Arcx](https://github.com/name-q/Arcx) framework.

## Quick Start

```bash
# 开发模式
arcx dev

# 生成 controller
arcx g c article

# 生成 service
arcx g s article

# 构建发布
cargo build --release
```
"#
    );
    fs::write(project_path.join("README.md"), readme).unwrap();

    println!("  ✓ Created project structure");
    println!("  ✓ Created config files");
    println!("  ✓ Created example controller");
    println!();
    println!("  Next steps:");
    println!();
    println!("    cd {}", name);
    println!("    arcx dev");
    println!();
}

// ─────────────────────────────────────────
// arcx generate controller <name>
// ─────────────────────────────────────────

fn cmd_generate_controller(name: &str) {
    ensure_in_project();
    let path = format!("src/controller/{}.rs", name);
    if Path::new(&path).exists() {
        eprintln!("✗ Controller '{}' already exists at {}", name, path);
        std::process::exit(1);
    }

    let struct_name = to_pascal_case(name);
    let content = format!(
        r#"//! {struct_name} Controller
//! 路由前缀: /api/{name}

use axum::{{
    extract::{{Path, State}},
    routing::{{get, post}},
    Json, Router,
}};
use serde_json::{{json, Value}};

use crate::context::AppState;
use crate::error::{{AppResult, success}};

/// 公开路由
pub fn routes() -> Router<AppState> {{
    Router::new()
        .route("/", get(list))
        .route("/:id", get(detail))
}}

/// 受保护路由（需要登录）
pub fn protected_routes() -> Router<AppState> {{
    Router::new()
        .route("/", post(create))
}}

/// GET /api/{name}
async fn list() -> AppResult<Json<Value>> {{
    Ok(success(json!({{
        "items": [],
        "total": 0
    }})))
}}

/// GET /api/{name}/:id
async fn detail(Path(id): Path<u64>) -> AppResult<Json<Value>> {{
    Ok(success(json!({{
        "id": id,
        "message": "{name} detail"
    }})))
}}

/// POST /api/{name}
async fn create(
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {{
    Ok(success(json!({{
        "message": "{name} created",
        "data": body
    }})))
}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, &content).unwrap();
    println!("✓ Created: {}", path);
    println!();
    println!("  Register in src/controller/mod.rs:");
    println!("    pub mod {};", name);
    println!();
    println!("  Register in src/router/mod.rs:");
    println!("    register_controllers!({}, ...);", name);
    println!("    register_protected_controllers!(state, {}, ...);", name);
}

// ─────────────────────────────────────────
// arcx generate service <name>
// ─────────────────────────────────────────

fn cmd_generate_service(name: &str) {
    ensure_in_project();
    let path = format!("src/service/{}.rs", name);
    if Path::new(&path).exists() {
        eprintln!("✗ Service '{}' already exists at {}", name, path);
        std::process::exit(1);
    }

    let struct_name = to_pascal_case(name);
    let content = format!(
        r#"//! {struct_name} Service
//!
//! 约定：
//! - Service 封装具体的业务逻辑
//! - Controller 调用 Service，Service 调用 Model/外部接口
//! - Service 之间可以互相调用

use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct {struct_name}Service {{
    db: Arc<DatabaseConnection>,
}}

impl {struct_name}Service {{
    pub fn new(db: Arc<DatabaseConnection>) -> Self {{
        Self {{ db }}
    }}

    /// 查询列表
    pub async fn find_all(&self) -> Result<Vec<serde_json::Value>, String> {{
        // TODO: 实现查询逻辑
        Ok(vec![])
    }}

    /// 根据 ID 查询
    pub async fn find_by_id(&self, id: u64) -> Result<Option<serde_json::Value>, String> {{
        // TODO: 实现查询逻辑
        Ok(None)
    }}
}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, content).unwrap();
    println!("✓ Created: {}", path);
    println!();
    println!("  Register in src/service/mod.rs:");
    println!("    pub mod {};", name);
}

// ─────────────────────────────────────────
// arcx generate model <name>
// ─────────────────────────────────────────

fn cmd_generate_model(name: &str) {
    ensure_in_project();
    let path = format!("src/model/{}.rs", name);
    if Path::new(&path).exists() {
        eprintln!("✗ Model '{}' already exists at {}", name, path);
        std::process::exit(1);
    }

    let struct_name = to_pascal_case(name);
    let content = format!(
        r#"//! {struct_name} Model
//!
//! 对应数据库表: {name}

use sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{name}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, content).unwrap();
    println!("✓ Created: {}", path);
    println!();
    println!("  Register in src/model/mod.rs:");
    println!("    pub mod {};", name);
}

// ─────────────────────────────────────────
// arcx generate job <name>
// ─────────────────────────────────────────

fn cmd_generate_job(name: &str) {
    ensure_in_project();
    let path = format!("src/schedule/{}.rs", name);
    if Path::new(&path).exists() {
        eprintln!("✗ Job '{}' already exists at {}", name, path);
        std::process::exit(1);
    }

    let struct_name = to_pascal_case(name);
    let content = format!(
        r#"//! {struct_name} 定时任务
//!
//! 约定：
//! - 实现 ScheduleJob trait
//! - cron 表达式为 6 位（秒 分 时 日 月 周）
//! - 通过 JobContext 访问共享资源

use crate::schedule::{{JobContext, ScheduleJob}};

pub struct {struct_name}Job;

#[async_trait::async_trait]
impl ScheduleJob for {struct_name}Job {{
    fn name(&self) -> &str {{
        "{name}"
    }}

    fn cron(&self) -> &str {{
        "0 */5 * * * *" // 每5分钟执行一次
    }}

    async fn run(&self, _ctx: &JobContext) {{
        tracing::info!("[{struct_name}Job] executing");
        // TODO: 实现任务逻辑
    }}
}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, content).unwrap();
    println!("✓ Created: {}", path);
    println!();
    println!("  Register schedule job in main.rs:");
    println!("    schedule_manager.register({struct_name}Job);");
}

// ─────────────────────────────────────────
// arcx dev
// ─────────────────────────────────────────

fn cmd_dev(port: u16) {
    println!("⚡ Starting Arcx dev server on port {}...", port);
    println!();

    // 检查是否有 cargo-watch
    let has_watch = Command::new("cargo")
        .args(["watch", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_watch {
        println!("  Using cargo-watch for hot reload");
        println!("  Watching src/ and config/ for changes...");
        println!();

        let status = Command::new("cargo")
            .args([
                "watch",
                "-x",
                "run --bin arcx-server",
                "-w",
                "src",
                "-w",
                "config",
            ])
            .env("ARCX_PORT", port.to_string())
            .status()
            .expect("Failed to start cargo-watch");

        std::process::exit(status.code().unwrap_or(1));
    } else {
        println!("  Tip: Install cargo-watch for hot reload:");
        println!("    cargo install cargo-watch");
        println!();

        let status = Command::new("cargo")
            .args(["run", "--bin", "arcx-server"])
            .env("ARCX_PORT", port.to_string())
            .status()
            .expect("Failed to start server");

        std::process::exit(status.code().unwrap_or(1));
    }
}

// ─────────────────────────────────────────
// arcx info
// ─────────────────────────────────────────

fn cmd_info() {
    ensure_in_project();

    // 读取 Cargo.toml 提取信息
    let cargo_content = fs::read_to_string("Cargo.toml").unwrap_or_default();
    let mut project_name = "unknown".to_string();
    let mut version = "unknown".to_string();

    for line in cargo_content.lines() {
        if line.starts_with("name") {
            project_name = line
                .split('=')
                .nth(1)
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_string();
        }
        if line.starts_with("version") && version == "unknown" {
            version = line
                .split('=')
                .nth(1)
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_string();
        }
    }

    println!("  Project: {}", project_name);
    println!("  Version: {}", version);
    println!();

    // 统计文件数
    let controllers = count_rs_files("src/controller");
    let services = count_rs_files("src/service");
    let models = count_rs_files("src/model");
    let middleware = count_rs_files("src/middleware");

    println!("  Controllers: {}", controllers);
    println!("  Services:    {}", services);
    println!("  Models:      {}", models);
    println!("  Middleware:   {}", middleware);
    println!();

    // 检查配置文件
    let config_dir = Path::new("config");
    if config_dir.exists() {
        println!("  Config files:");
        if let Ok(entries) = fs::read_dir(config_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                println!("    - {}", name.to_string_lossy());
            }
        }
    }
}

// ─────────────────────────────────────────
// 工具函数
// ─────────────────────────────────────────

/// 确保文件的父目录存在
fn ensure_parent(path: &str) {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).ok();
    }
}

/// 确保在项目目录内执行
fn ensure_in_project() {
    if !Path::new("Cargo.toml").exists() {
        eprintln!("✗ Not in a Rust project directory (no Cargo.toml found)");
        eprintln!("  Run this command from your project root.");
        std::process::exit(1);
    }
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

/// 统计目录下 .rs 文件数量（排除 mod.rs）
fn count_rs_files(dir: &str) -> usize {
    let path = Path::new(dir);
    if !path.exists() {
        return 0;
    }
    fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.ends_with(".rs") && name != "mod.rs"
                })
                .count()
        })
        .unwrap_or(0)
}
