//! Arcx CLI 脚手架工具
//!
//! 命令：
//! - arcx new <project>              创建新项目
//! - arcx generate controller <name> 生成 controller
//! - arcx generate service <name>    生成 service
//! - arcx generate model <name>      生成 model
//! - arcx generate job <name>        生成定时任务
//! - arcx dev                        启动开发服务器（热重载）
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
    let dirs = ["src/controller", "src/service", "src/model", "src/middleware", "config"];
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
    fs::write(
        project_path.join("config/config.default.toml"),
        default_config,
    )
    .unwrap();

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
        r#"//! {project_name} — Powered by Arcx

mod controller;
mod helper;
mod middleware;
mod router;

use arcx_core::prelude::*;
// use crate::middleware::auth::JwtAuth;

#[tokio::main]
async fn main() {{
    Arcx::new()
        // 开启鉴权后取消注释，guarded_scope 路由会自动调用 authenticate
        // .auth(JwtAuth::new("your-secret-key"))
        .routes(router::routes)
        .run()
        .await;
}}
"#
    );
    fs::write(project_path.join("src/main.rs"), main_rs).unwrap();

    // src/helper.rs
    let helper_rs = r#"//! 响应 Helper — 项目的响应格式约定，按需修改
//!
//! 这是你的项目代码，框架不依赖它。你可以：
//! - 修改 JSON 结构
//! - 添加自己的响应方法
//! - 或者完全不用它，直接返回 axum 原生类型

#![allow(dead_code)]

use arcx_core::prelude::*;

/// 成功响应（200）
pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
    Json(json!({
        "code": 0,
        "data": data,
        "message": "success"
    }))
}

/// 成功响应 + 自定义消息
pub fn success_msg<T: Serialize>(data: T, msg: &str) -> impl IntoResponse {
    Json(json!({
        "code": 0,
        "data": data,
        "message": msg
    }))
}

/// 创建成功（201）
pub fn created<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({
        "code": 0,
        "data": data,
        "message": "created"
    })))
}

/// 无内容（204）
pub fn no_content() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

/// 分页响应
pub fn paginate<T: Serialize>(list: Vec<T>, total: u64, page: u64, page_size: u64) -> impl IntoResponse {
    Json(json!({
        "code": 0,
        "data": {
            "list": list,
            "total": total,
            "page": page,
            "page_size": page_size
        }
    }))
}

/// 业务失败（200 但 code != 0）
pub fn fail(code: i32, msg: &str) -> impl IntoResponse {
    Json(json!({
        "code": code,
        "message": msg
    }))
}
"#;
    fs::write(project_path.join("src/helper.rs"), helper_rs).unwrap();

    // src/router.rs
    let router_rs = r#"//! 路由声明 — 自由组合

use arcx_core::prelude::*;
use crate::controller;

pub fn routes(r: &mut ArcxRouter) {
    // 公开路由
    r.get("/api/home", controller::home::index);
    r.get("/api/home/:id", controller::home::show);
    r.post("/api/home", controller::home::create);
    r.put("/api/home/:id", controller::home::update);
    r.delete("/api/home/:id", controller::home::destroy);

    // 鉴权路由 — 需要先在 main.rs 中注册 .auth(provider)
    // r.guarded_scope("/api/admin", |s| {
    //     s.get("/profile", controller::admin::profile);
    // });
}
"#;
    fs::write(project_path.join("src/router.rs"), router_rs).unwrap();

    // src/controller/mod.rs
    fs::write(project_path.join("src/controller/mod.rs"), "pub mod home;\n").unwrap();

    // src/controller/home.rs
    let home_controller = format!(
        r#"//! Home Controller

use arcx_core::prelude::*;
use crate::helper;

/// GET /api/home
pub async fn index(ctx: Context) -> AppResult<impl IntoResponse> {{
    Ok(helper::success(json!({{
        "name": ctx.config.app.name,
        "version": ctx.config.app.version,
        "message": "Welcome to Arcx!"
    }})))
}}

/// GET /api/home/:id
pub async fn show(_ctx: Context, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {{
    Ok(helper::success(json!({{ "id": id }})))
}}

/// POST /api/home
pub async fn create(_ctx: Context, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {{
    Ok(helper::created(json!({{ "item": body }})))
}}

/// PUT /api/home/:id
pub async fn update(_ctx: Context, Path(id): Path<u64>, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {{
    Ok(helper::success(json!({{ "id": id, "updated": body }})))
}}

/// DELETE /api/home/:id
pub async fn destroy(_ctx: Context, Path(_id): Path<u64>) -> AppResult<impl IntoResponse> {{
    Ok(helper::no_content())
}}
"#
    );
    fs::write(
        project_path.join("src/controller/home.rs"),
        home_controller,
    )
    .unwrap();

    // src/service/mod.rs
    fs::write(
        project_path.join("src/service/mod.rs"),
        "// Service 层：封装业务逻辑\n",
    )
    .unwrap();

    // src/middleware/mod.rs
    fs::write(
        project_path.join("src/middleware/mod.rs"),
        "pub mod auth;\n",
    )
    .unwrap();

    // src/middleware/auth.rs — 鉴权实现示例（用户代码，按需修改）
    let auth_middleware = r#"//! 鉴权实现 — 按需修改
//!
//! 实现 AuthProvider trait，控制 token 从哪取、怎么验证。
//! 框架不绑定任何具体方案，你可以用 JWT、Session、OAuth 等。

#![allow(dead_code)]

use arcx_core::prelude::*;

/// 你的鉴权提供者
pub struct JwtAuth {
    secret: String,
}

impl JwtAuth {
    pub fn new(secret: impl Into<String>) -> Self {
        Self { secret: secret.into() }
    }
}

#[async_trait]
impl AuthProvider for JwtAuth {
    async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError> {
        // 1. 从 Header 取 token（可改为 cookie、query 等）
        let token = parts.headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::unauthorized("Missing Authorization header"))?;

        // 2. 验证 token（替换为你的验证逻辑）
        // 示例：使用 JWT 插件
        // let jwt = state.resource::<JwtService>().unwrap();
        // let claims = jwt.verify(token).map_err(|_| AppError::unauthorized("Invalid token"))?;

        // 临时示例：接受任何非空 token
        if token.is_empty() {
            return Err(AppError::unauthorized("Empty token"));
        }

        Ok(AuthUser {
            id: "user_from_token".to_string(),
            payload: serde_json::json!({ "role": "user" }),
        })
    }
}
"#;
    fs::write(project_path.join("src/middleware/auth.rs"), auth_middleware).unwrap();


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

# Hot reload
arcx dev

# Generate code
arcx g c user
arcx g s user
```

## Project Structure

```
src/
├── main.rs           # Entry point
├── router.rs         # Route declarations (free style)
├── helper.rs         # Response helpers (customizable)
├── controller/       # Handler functions
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

use arcx_core::prelude::*;
use crate::helper;

/// GET /api/{name}
pub async fn index(_ctx: Context) -> AppResult<impl IntoResponse> {{
    Ok(helper::success(json!({{ "items": [], "total": 0 }})))
}}

/// GET /api/{name}/:id
pub async fn show(_ctx: Context, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {{
    Ok(helper::success(json!({{ "id": id }})))
}}

/// POST /api/{name}
pub async fn create(_ctx: Context, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {{
    Ok(helper::created(json!({{ "item": body }})))
}}

/// PUT /api/{name}/:id
pub async fn update(_ctx: Context, Path(id): Path<u64>, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {{
    Ok(helper::success(json!({{ "id": id, "updated": body }})))
}}

/// DELETE /api/{name}/:id
pub async fn destroy(_ctx: Context, Path(_id): Path<u64>) -> AppResult<impl IntoResponse> {{
    Ok(helper::no_content())
}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, &content).unwrap();
    println!("✓ Created: {}", path);

    // 注册到 mod.rs
    auto_register_mod("src/controller/mod.rs", name);

    // 注册到 router.rs
    auto_register_in_router(name);

    println!("✓ Auto-registered in mod.rs and router.rs");
    println!("  Route: /api/{}", name);
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

use arcx_core::prelude::*;

pub struct {struct_name}Service;

impl {struct_name}Service {{
    pub fn new() -> Self {{
        Self
    }}

    pub async fn find_all(&self) -> AppResult<Vec<Value>> {{
        Ok(vec![])
    }}

    pub async fn find_by_id(&self, _id: u64) -> AppResult<Option<Value>> {{
        Ok(None)
    }}

    pub async fn create(&self, _data: Value) -> AppResult<Value> {{
        Ok(json!({{}}))
    }}
}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, content).unwrap();
    println!("✓ Created: {}", path);

    auto_register_mod("src/service/mod.rs", name);
    println!("✓ Auto-registered in service/mod.rs");
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

    auto_register_mod("src/model/mod.rs", name);
    println!("✓ Auto-registered in model/mod.rs");
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
// Auto-registration helpers
// ─────────────────────────────────────────

/// 向 mod.rs 中追加 `pub mod <name>;`
fn auto_register_mod(mod_path: &str, name: &str) {
    let mod_file = Path::new(mod_path);
    if !mod_file.exists() {
        fs::write(mod_file, format!("pub mod {};\n", name)).unwrap();
        return;
    }

    let content = fs::read_to_string(mod_file).unwrap();
    let mod_line = format!("pub mod {};", name);

    if content.lines().any(|l| l.trim() == mod_line) {
        return;
    }

    let mut new_content = content.trim_end().to_string();
    new_content.push_str(&format!("\npub mod {};\n", name));
    fs::write(mod_file, new_content).unwrap();
}

/// 在 router.rs 中追加自由路由声明
fn auto_register_in_router(name: &str) {
    let router_path = Path::new("src/router.rs");
    if !router_path.exists() {
        eprintln!("  ⚠ src/router.rs not found, skip auto-register");
        return;
    }

    let content = fs::read_to_string(router_path).unwrap();

    // 检查是否已注册
    if content.contains(&format!("controller::{}::", name)) {
        return;
    }

    // 找到 routes 函数的最后一个 } 之前插入
    if let Some(last_brace) = content.rfind('}') {
        let new_lines = format!(
            "\n    // {name} routes\n    r.get(\"/api/{name}\", controller::{name}::index);\n    r.get(\"/api/{name}/:id\", controller::{name}::show);\n    r.post(\"/api/{name}\", controller::{name}::create);\n    r.put(\"/api/{name}/:id\", controller::{name}::update);\n    r.delete(\"/api/{name}/:id\", controller::{name}::destroy);\n",
        );
        let new_content =
            format!("{}{}{}", &content[..last_brace], new_lines, &content[last_brace..]);
        fs::write(router_path, new_content).unwrap();
    } else {
        eprintln!("  ⚠ Could not parse router.rs, skip auto-register");
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
