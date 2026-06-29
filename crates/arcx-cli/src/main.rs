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
mod dev;
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
        Commands::Dev { port } => dev::run(port),
        Commands::Info => cmd_info(),
    }
}

// ─────────────────────────────────────────
// arcx new <project>
// ─────────────────────────────────────────

fn cmd_new(name: &str) {
    let project_path = Path::new(name);
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
arcx-core = "0.1"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tracing = "0.1"
validator = {{ version = "0.18", features = ["derive"] }}
async-trait = "0.1"
axum = {{ version = "0.7", features = ["ws"] }}
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
security = true

[logger]
level = "info"
enable_console = true
enable_file = false

[httpclient]
timeout = 30
max_retries = 0

[security]
csrf = false

[schedule]
enable = false

# [plugin.database]
# enable = true
# url = "sqlite:./data.db?mode=rwc"

# [plugin.jwt]
# enable = true
# secret = "change-me-in-production-at-least-32-chars"
# expire = 86400
"#
    );
    fs::write(project_path.join("config/config.default.toml"), default_config).unwrap();

    // config/config.prod.toml
    let prod_config = r#"[app]
env = "prod"

[server]
host = "0.0.0.0"
port = 8080

[middleware]
cors = false

[logger]
level = "info"
enable_file = true
dir = "logs"

[security]
csrf = true
hsts = true
"#;
    fs::write(project_path.join("config/config.prod.toml"), prod_config).unwrap();

    // src/main.rs
    let main_rs = format!(
        r#"//! {project_name} — Powered by Arcx Framework

mod controller;
mod service;

use arcx_core::prelude::*;

#[tokio::main]
async fn main() {{
    // 1. 加载配置
    let cfg = Arcx::load_config();

    // 2. 初始化日志
    arcx_core::logger::init(&cfg.logger);
    tracing::info!("{{}} v{{}} starting...", cfg.app.name, cfg.app.version);

    // 3. 加载插件
    let raw_config = Arcx::load_raw_config();
    let mut plugin_manager = PluginManager::new();
    plugin_manager.auto_register_builtin(&raw_config);
    if let Err(e) = plugin_manager.init_all(&raw_config).await {{
        tracing::error!("Plugin init failed: {{}}", e);
        std::process::exit(1);
    }}

    // 4. 构建 HttpClient
    let http_client = HttpClient::new(cfg.httpclient.clone());

    // 5. 构建事件总线 & 配置热更新
    let event_bus = EventBus::new(128);
    let (_notifier, config_watcher) = ConfigWatcher::new(cfg.clone());

    // 6. 构建共享状态
    let mut resources = plugin_manager.take_resources();
    resources.insert(
        std::any::TypeId::of::<HttpClient>(),
        std::sync::Arc::new(http_client),
    );
    let state = AppState::with_resources(cfg.clone(), resources, event_bus.clone(), config_watcher);

    // 7. 构建路由
    let public = arcx_core::register_controllers!(AppState, controller, home);
    let api = axum::Router::new().nest("/api", public);
    let app = apply_global_middleware(api, &cfg).with_state(state);

    // 8. 启动服务
    let addr = format!("{{}}:{{}}", cfg.server.host, cfg.server.port);
    tracing::info!("Server running at http://{{}}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(async {{
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("Shutting down...");
        }})
        .await
        .unwrap();

    plugin_manager.shutdown_all().await;
}}
"#
    );
    fs::write(project_path.join("src/main.rs"), main_rs).unwrap();

    // src/controller/mod.rs
    let controller_mod = r#"pub mod home;
"#;
    fs::write(project_path.join("src/controller/mod.rs"), controller_mod).unwrap();

    // src/controller/home.rs
    let home_controller = format!(
        r#"use axum::{{routing::get, Json, Router}};
use arcx_core::prelude::*;

pub fn routes() -> Router<AppState> {{
    Router::new()
        .route("/", get(index))
}}

async fn index(ctx: Context) -> Json<serde_json::Value> {{
    Json(json!({{
        "name": ctx.config.app.name,
        "version": ctx.config.app.version,
        "message": "Welcome to Arcx!"
    }}))
}}
"#
    );
    fs::write(project_path.join("src/controller/home.rs"), home_controller).unwrap();

    // src/service/mod.rs
    fs::write(
        project_path.join("src/service/mod.rs"),
        "// Service 层：封装业务逻辑\n",
    )
    .unwrap();

    // .gitignore
    let gitignore = "target/\n*.db\n.env\n.DS_Store\nlogs/\n";
    fs::write(project_path.join(".gitignore"), gitignore).unwrap();

    // README.md
    let readme = format!(
        r#"# {project_name}

Built with [Arcx](https://github.com/name-q/Arcx) framework.

## Quick Start

```bash
cargo run
```

## Development

```bash
# Install CLI
cargo install arcx-cli

# Generate code
arcx g c article
arcx g s article

# Hot reload (requires cargo-watch)
arcx dev
```

## Project Structure

```
src/
├── main.rs           # Entry point
├── controller/       # Route handlers (one file per resource)
├── service/          # Business logic
└── model/            # Database entities
config/
├── config.default.toml
└── config.prod.toml
```
"#
    );
    fs::write(project_path.join("README.md"), readme).unwrap();

    println!("  ✓ Project structure created");
    println!("  ✓ Config files generated");
    println!("  ✓ Example controller ready");
    println!();
    println!("  Next steps:");
    println!();
    println!("    cd {}", name);
    println!("    cargo run");
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
//! Routes: /api/{name}

use axum::{{
    extract::Path,
    routing::{{get, post}},
    Json, Router,
}};
use arcx_core::prelude::*;

/// Public routes
pub fn routes() -> Router<AppState> {{
    Router::new()
        .route("/", get(list))
        .route("/:id", get(detail))
}}

/// Protected routes (require auth)
pub fn protected_routes() -> Router<AppState> {{
    Router::new()
        .route("/", post(create))
}}

async fn list() -> AppResult<Json<serde_json::Value>> {{
    Ok(success(json!({{ "items": [], "total": 0 }})))
}}

async fn detail(Path(id): Path<u64>) -> AppResult<Json<serde_json::Value>> {{
    Ok(success(json!({{ "id": id }})))
}}

async fn create(Json(body): Json<serde_json::Value>) -> AppResult<Json<serde_json::Value>> {{
    Ok(success(json!({{ "created": body }})))
}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, &content).unwrap();
    println!("✓ Created: {}", path);
    println!();
    println!("  Register in src/controller/mod.rs:");
    println!("    pub mod {};", name);
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

use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct {struct_name}Service {{
    db: Arc<DatabaseConnection>,
}}

impl {struct_name}Service {{
    pub fn new(db: Arc<DatabaseConnection>) -> Self {{
        Self {{ db }}
    }}

    pub async fn find_all(&self) -> Result<Vec<serde_json::Value>, String> {{
        Ok(vec![])
    }}

    pub async fn find_by_id(&self, _id: u64) -> Result<Option<serde_json::Value>, String> {{
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
        r#"//! {struct_name} Entity (SeaORM)

use sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{name}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
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
        r#"//! {struct_name} Job

use arcx_core::prelude::*;

pub struct {struct_name}Job;

#[async_trait]
impl ScheduleJob for {struct_name}Job {{
    fn name(&self) -> &str {{
        "{name}"
    }}

    fn cron(&self) -> &str {{
        "0 */5 * * * *" // every 5 minutes
    }}

    async fn run(&self, _ctx: &JobContext) {{
        tracing::info!("[{struct_name}Job] executing");
    }}
}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, content).unwrap();
    println!("✓ Created: {}", path);
    println!();
    println!("  Register in main.rs:");
    println!("    schedule_manager.register({}Job);", struct_name);
}


// ─────────────────────────────────────────
// arcx info
// ─────────────────────────────────────────

fn cmd_info() {
    ensure_in_project();

    let cargo_content = fs::read_to_string("Cargo.toml").unwrap_or_default();
    let mut project_name = "unknown".to_string();
    let mut version = "unknown".to_string();

    for line in cargo_content.lines() {
        if line.starts_with("name") {
            project_name = line.split('=').nth(1).unwrap_or("").trim().trim_matches('"').to_string();
        }
        if line.starts_with("version") && version == "unknown" {
            version = line.split('=').nth(1).unwrap_or("").trim().trim_matches('"').to_string();
        }
    }

    println!("  Project: {}", project_name);
    println!("  Version: {}", version);
    println!();

    let controllers = count_rs_files("src/controller");
    let services = count_rs_files("src/service");
    let models = count_rs_files("src/model");

    println!("  Controllers: {}", controllers);
    println!("  Services:    {}", services);
    println!("  Models:      {}", models);
    println!();

    if Path::new("config").exists() {
        println!("  Config files:");
        if let Ok(entries) = fs::read_dir("config") {
            for entry in entries.flatten() {
                println!("    - {}", entry.file_name().to_string_lossy());
            }
        }
    }
}

// ─────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────

fn ensure_parent(path: &str) {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).ok();
    }
}

fn ensure_in_project() {
    if !Path::new("Cargo.toml").exists() {
        eprintln!("✗ Not in a Rust project directory (no Cargo.toml found)");
        std::process::exit(1);
    }
}

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

fn count_rs_files(dir: &str) -> usize {
    let path = Path::new(dir);
    if !path.exists() { return 0; }
    fs::read_dir(path)
        .map(|entries| {
            entries.flatten()
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.ends_with(".rs") && name != "mod.rs"
                })
                .count()
        })
        .unwrap_or(0)
}
