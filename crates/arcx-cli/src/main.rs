//! Arcx CLI 脚手架工具
//!
//! 命令：
//! - arcx new <project>              创建新项目
//! - arcx generate controller <name> 生成 controller (alias: c)
//! - arcx generate service <name>    生成 service    (alias: s)
//! - arcx generate middleware <name> 生成 middleware  (alias: m)
//! - arcx generate job <name>        生成定时任务     (alias: j)
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
        #[arg(short, long, default_value = "8765")]
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
    /// 生成 middleware
    #[command(alias = "m")]
    Middleware { name: String },
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
            GenerateTarget::Middleware { name } => cmd_generate_middleware(&name),
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
        "src/middleware",
        "src/helper",
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
port = 8765

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

    // src/prelude.rs — 项目级 prelude，收敛所有常用 import
    let prelude_rs = r#"//! 项目 Prelude — 所有模块统一使用 `use crate::prelude::*;`

pub use arcx_core::prelude::*;
pub use crate::helper::*;
pub use crate::service::{ServiceAccess, *};
"#;
    fs::write(project_path.join("src/prelude.rs"), prelude_rs).unwrap();

    // src/main.rs
    let main_rs = r#"mod controller;
mod helper;
mod middleware;
mod prelude;
mod router;
mod service;

use crate::prelude::*;

#[tokio::main]
async fn main() {
    Arcx::new()
        .routes(router::routes)
        .run()
        .await;
}
"#;
    fs::write(project_path.join("src/main.rs"), main_rs).unwrap();

    // src/helper/mod.rs
    let helper_mod = r#"pub mod response;
"#;
    fs::write(project_path.join("src/helper/mod.rs"), helper_mod).unwrap();

    // src/helper/response.rs
    let helper_response = r#"//! 响应格式封装 — 按需修改

#![allow(dead_code)]

use arcx_core::prelude::*;

/// 成功响应
pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
    Json(json!({
        "code": 0,
        "data": data,
        "message": "success"
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

/// 业务失败
pub fn fail(code: i32, msg: &str) -> impl IntoResponse {
    Json(json!({
        "code": code,
        "message": msg
    }))
}
"#;
    fs::write(project_path.join("src/helper/response.rs"), helper_response).unwrap();

    // src/router.rs
    let router_rs = r#"use crate::prelude::*;
use crate::controller;
// use crate::middleware;

pub fn routes(r: &mut ArcxRouter) {
    // 全局中间件（可选）
    // r.middleware(middleware::log::handle);

    r.get("/api/home", controller::home::index);
    r.get("/api/home/:id", controller::home::show);
    r.post("/api/home", controller::home::create);

    // 路由级中间件示例
    // r.get("/api/protected", controller::home::index)
    //     .middleware(middleware::auth::handle);
}
"#;
    fs::write(project_path.join("src/router.rs"), router_rs).unwrap();

    // src/controller/mod.rs
    fs::write(project_path.join("src/controller/mod.rs"), "pub mod home;\n").unwrap();

    // src/controller/home.rs — 只有一行 use
    let home_controller = r#"use crate::prelude::*;

/// GET /api/home
pub async fn index(ctx: Ctx) -> AppResult<impl IntoResponse> {
    let name = &ctx.config().app.name;
    Ok(response::success(json!({
        "message": format!("Welcome to {}!", name)
    })))
}

/// GET /api/home/:id
pub async fn show(ctx: Ctx, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    let user = ctx.services().user.find_by_id(id).await?;
    Ok(response::success(user))
}

/// POST /api/home
pub async fn create(Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(response::created(json!({ "item": body })))
}
"#;
    fs::write(project_path.join("src/controller/home.rs"), home_controller).unwrap();

    // src/service/mod.rs — 带一个默认 user service
    let service_mod = r#"arcx_core::services! {
    user: UserService,
}
"#;
    fs::write(project_path.join("src/service/mod.rs"), service_mod).unwrap();

    // src/service/user.rs
    let user_service = r#"use crate::prelude::*;

#[service]
impl UserService {
    pub async fn find_by_id(&self, id: u64) -> AppResult<Value> {
        Ok(json!({ "id": id, "name": format!("User_{}", id) }))
    }
}
"#;
    fs::write(project_path.join("src/service/user.rs"), user_service).unwrap();

    // src/middleware/mod.rs
    fs::write(project_path.join("src/middleware/mod.rs"), "").unwrap();

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
cargo install arcx-cli
arcx dev
arcx g c user    # generate controller
arcx g s user    # generate service
```
"#
    );
    fs::write(project_path.join("README.md"), readme).unwrap();

    println!("  ✓ Project created");
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

    let content = format!(
        r#"use crate::prelude::*;

/// GET /api/{name}
pub async fn index() -> AppResult<impl IntoResponse> {{
    Ok(response::success(json!({{ "items": [], "total": 0 }})))
}}

/// GET /api/{name}/:id
pub async fn show(ctx: Ctx, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {{
    Ok(response::success(json!({{ "id": id }})))
}}

/// POST /api/{name}
pub async fn create(Json(body): Json<Value>) -> AppResult<impl IntoResponse> {{
    Ok(response::created(json!({{ "item": body }})))
}}

/// PUT /api/{name}/:id
pub async fn update(Path(id): Path<u64>, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {{
    Ok(response::success(json!({{ "id": id, "updated": body }})))
}}

/// DELETE /api/{name}/:id
pub async fn destroy(Path(_id): Path<u64>) -> AppResult<impl IntoResponse> {{
    Ok(response::no_content())
}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, &content).unwrap();
    println!("✓ Created: {}", path);

    auto_register_mod("src/controller/mod.rs", name);
    auto_register_in_router(name);
    println!("✓ Registered in mod.rs + router.rs");
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
        r#"use crate::prelude::*;

#[service]
impl {struct_name}Service {{
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

    auto_register_service("src/service/mod.rs", name, &to_pascal_case(name));
    println!("✓ Registered in services! {{}}");
}


// ─────────────────────────────────────────
// arcx generate middleware <name>
// ─────────────────────────────────────────

fn cmd_generate_middleware(name: &str) {
    ensure_in_project();
    let path = format!("src/middleware/{}.rs", name);
    if Path::new(&path).exists() {
        eprintln!("✗ Middleware '{}' already exists at {}", name, path);
        std::process::exit(1);
    }

    let content = format!(
        r#"use crate::prelude::*;

/// {name} 中间件
pub async fn handle(ctx: Ctx, next: Next, parts: ReqParts) -> Response {{
    // TODO: 前置逻辑（可用 ctx.header() / ctx.config() / ctx.services() 等）

    // 放行到下一层
    let response = ctx.next(next, parts).await;

    // TODO: 后置逻辑

    response
}}
"#
    );

    ensure_parent(&path);
    fs::write(&path, content).unwrap();
    println!("✓ Created: {}", path);

    auto_register_mod("src/middleware/mod.rs", name);
    println!("✓ Registered in middleware/mod.rs");
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
        r#"use crate::prelude::*;

pub struct {struct_name}Job;

#[async_trait]
impl ScheduleJob for {struct_name}Job {{
    fn name(&self) -> &str {{
        "{name}"
    }}

    fn cron(&self) -> &str {{
        "0 */5 * * * *"
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
    println!("  Register in main.rs: schedule_manager.register({}Job);", struct_name);
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
    let helpers = count_rs_files("src/helper");
    let middlewares = count_rs_files("src/middleware");

    println!("  Controllers:  {}", controllers);
    println!("  Services:     {}", services);
    println!("  Helpers:      {}", helpers);
    println!("  Middlewares:  {}", middlewares);
}

// ─────────────────────────────────────────
// Auto-registration helpers
// ─────────────────────────────────────────

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

fn auto_register_service(mod_path: &str, name: &str, struct_name: &str) {
    let mod_file = Path::new(mod_path);
    if !mod_file.exists() {
        let content = format!(
            "arcx_core::services! {{\n    {}: {}Service,\n}}\n",
            name, struct_name
        );
        fs::write(mod_file, content).unwrap();
        return;
    }

    let content = fs::read_to_string(mod_file).unwrap();

    let entry = format!("{}: {}Service", name, struct_name);
    if content.contains(&entry) {
        return;
    }

    if content.contains("arcx_core::services!") {
        if let Some(pos) = content.rfind('}') {
            let new_content = format!(
                "{}    {}: {}Service,\n{}",
                &content[..pos], name, struct_name, &content[pos..]
            );
            fs::write(mod_file, new_content).unwrap();
        }
    } else {
        let content = format!(
            "arcx_core::services! {{\n    {}: {}Service,\n}}\n",
            name, struct_name
        );
        fs::write(mod_file, content).unwrap();
    }
}

fn auto_register_in_router(name: &str) {
    let router_path = Path::new("src/router.rs");
    if !router_path.exists() {
        return;
    }

    let content = fs::read_to_string(router_path).unwrap();
    if content.contains(&format!("controller::{}::", name)) {
        return;
    }

    if let Some(last_brace) = content.rfind('}') {
        let new_lines = format!(
            "\n    // {name}\n    r.get(\"/api/{name}\", controller::{name}::index);\n    r.get(\"/api/{name}/:id\", controller::{name}::show);\n    r.post(\"/api/{name}\", controller::{name}::create);\n    r.put(\"/api/{name}/:id\", controller::{name}::update);\n    r.delete(\"/api/{name}/:id\", controller::{name}::destroy);\n",
        );
        let new_content =
            format!("{}{}{}", &content[..last_brace], new_lines, &content[last_brace..]);
        fs::write(router_path, new_content).unwrap();
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
