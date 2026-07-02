# Arcx

[![Crates.io](https://img.shields.io/crates/v/arcx-core?label=arcx-core)](https://crates.io/crates/arcx-core)
[![Crates.io](https://img.shields.io/crates/v/arcx-cli?label=arcx-cli)](https://crates.io/crates/arcx-cli)
[![GitHub](https://img.shields.io/badge/GitHub-name--q%2FArcx-blue?logo=github)](https://github.com/name-q/Arcx)

A convention-over-configuration web framework for Rust, built on [Axum](https://github.com/tokio-rs/axum).

Arc(Architecture) + X(Extensible) — write business logic, not boilerplate.

## Quick Start

```bash
cargo install arcx-cli
arcx new my-app
cd my-app
cargo run
```

## What it looks like

```rust
// src/main.rs
use arcx_core::prelude::*;

#[tokio::main]
async fn main() {
    Arcx::new()
        .routes(router::routes)
        .run()
        .await;
}
```

```rust
// src/router.rs
pub fn routes(r: &mut ArcxRouter) {
    r.get("/api/users/:id", controller::user::show);
    r.post("/api/users", controller::user::create);

    r.guarded_scope("/api/admin", |s| {
        s.get("/dashboard", controller::admin::dashboard);
    });
}
```

```rust
// src/controller/user.rs
use crate::prelude::*;

pub async fn show(ctx: Ctx, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    let user = ctx.services().user.find_by_id(id).await?;
    Ok(response::success(user))
}
```

```rust
// src/service/user.rs
use crate::prelude::*;

#[service]
impl UserService {
    pub async fn find_by_id(&self, id: u64) -> AppResult<Value> {
        Ok(json!({ "id": id, "name": format!("User_{}", id) }))
    }
}
```

## Project Structure

```
my-app/
├── src/
│   ├── main.rs
│   ├── router.rs
│   ├── prelude.rs
│   ├── controller/
│   │   └── home.rs
│   ├── service/
│   │   ├── mod.rs       # services! { user: UserService }
│   │   └── user.rs
│   ├── middleware/
│   ├── schedule/
│   └── helper/
│       └── response.rs  # Your response format (customizable)
├── config/
│   ├── config.default.toml
│   └── config.prod.toml
└── Cargo.toml
```

## Core Concepts

### Services Container

Register once, use everywhere via `ctx.services()`:

```rust
// src/service/mod.rs
arcx_core::services! {
    user: UserService,
    order: OrderService,
}
```

The `#[service]` macro generates the struct and wiring — you only write methods:

```rust
// src/service/order.rs
use crate::prelude::*;

#[service]
impl OrderService {
    pub async fn find_by_user(&self, user_id: &str) -> AppResult<Value> {
        // business logic here
    }
}
```

Services can call each other:

```rust
#[service]
impl UserService {
    pub async fn find_with_orders(&self, id: &str) -> AppResult<Value> {
        let user = self.find_by_id(id).await?;
        let orders = self.ctx.service::<OrderService>().find_by_user(id).await?;
        Ok(json!({ "user": user, "orders": orders }))
    }
}
```

### Routing

Free-style routing — no macros, no forced conventions:

```rust
pub fn routes(r: &mut ArcxRouter) {
    // Global middleware
    r.middleware(middleware::log::handle);

    // Standard routes
    r.get("/api/home", controller::home::index);
    r.post("/api/home", controller::home::create);

    // Route-level middleware (chainable)
    r.get("/api/admin", controller::admin::dashboard)
        .middleware(middleware::auth::handle);

    // Scoped middleware
    r.scope("/api/v2", |s| {
        s.middleware(middleware::auth::handle);
        s.get("/users", controller::user::list);
    });

    // Protected routes (requires .auth() in main.rs)
    r.guarded_scope("/api/admin", |s| {
        s.get("/profile", controller::admin::profile);
    });
}
```

### Middleware

Middleware and Controller share the same `Ctx`, same API:

```rust
use crate::prelude::*;

pub async fn handle(ctx: Ctx, next: Next, parts: ReqParts) -> Response {
    let start = std::time::Instant::now();

    // Access config, services, headers — same as controller
    let env = ctx.env();

    // Inject data for downstream
    ctx.set(MyData { role: "admin".into() });

    // Pass through
    let response = ctx.next(next, parts).await;

    println!("Request took {:?}", start.elapsed());
    response
}
```

Controllers retrieve middleware-injected data via `ctx.get::<T>()`:

```rust
pub async fn dashboard(ctx: Ctx) -> AppResult<impl IntoResponse> {
    let data = ctx.get::<MyData>().unwrap();
    Ok(Json(json!({ "role": data.role })))
}
```

### Ctx — Request-scoped service locator

```rust
pub async fn show(ctx: Ctx, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    // Quick access methods
    let method = ctx.method();
    let path = ctx.path();
    let host = ctx.header("host");
    let ip = ctx.ip();
    let params: MyQuery = ctx.query()?;

    // Config access (dot notation)
    let port: Option<u16> = ctx.conf("server.port");
    let redis_url: Option<String> = ctx.conf("redis.url");

    // Services
    let user = ctx.services().user.find_by_id(id).await?;
    Ok(Json(user))
}
```

Ctx is optional — handlers that don't need it simply don't declare it:

```rust
pub async fn health() -> &'static str {
    "ok"
}
```

### Auth

Implement the `AuthProvider` trait with any strategy (JWT, session, OAuth):

```rust
pub struct JwtAuth { secret: String }

#[async_trait]
impl AuthProvider for JwtAuth {
    async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError> {
        let token = parts.headers.get("Authorization") /* ... */;
        Ok(AuthUser { id: "user_1".into(), payload: json!({}) })
    }
}
```

Enable it in main:

```rust
Arcx::new()
    .auth(JwtAuth::new("secret"))
    .routes(router::routes)
    .run()
    .await;
```

### Configuration

Multi-environment TOML. Each middleware/feature manages its own section:

```toml
# config/config.default.toml
[server]
port = 3000

[cors]
enable = true
allowed_origins = ["*"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allow_credentials = false
max_age = 86400

[request_logger]
enable = true

[security]
enable = true
csrf = false
xss_protection = true
frame_deny = true

[database]
url = "postgres://localhost/mydb"
```

Config files support `import` for splitting:
```toml
import = ["config/custom/redis.toml"]

[app]
name = "my-app"
```

### Schedule Jobs

Cron-based job scheduling:

```rust
// src/schedule/cleanup.rs
use arcx_core::prelude::*;

pub struct CleanupJob;

impl ScheduleJob for CleanupJob {
    fn name(&self) -> &str { "cleanup" }
    fn cron(&self) -> &str { "0 */5 * * * *" }  // every 5 minutes

    fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async {
            println!("[CleanupJob] executing");
        })
    }
}
```

Register in main:

```rust
Arcx::new()
    .routes(router::routes)
    .schedule(CleanupJob)
    .run()
    .await;
```

### Plugins

Database, caching, and custom plugins — register once, access via `ctx.plugin::<T>()`:

```rust
let db = ctx.plugin::<DatabasePlugin>()?;
```

## Features

| Category | Capabilities |
|----------|-------------|
| **Routing** | Free-style, guarded scopes, path params, nested scopes |
| **Middleware** | Global / scope / route-level, onion model, Ctx shared |
| **Services** | `#[service]` macro, container pattern, inter-service calls |
| **Auth** | Pluggable AuthProvider trait, route guards |
| **Config** | Multi-env TOML, dot-notation, import, hot reload |
| **Plugins** | Database (SeaORM), JWT, custom plugin trait |
| **Session** | HMAC-signed cookies, extensible store |
| **WebSocket** | Trait-based handler, session management |
| **Schedule** | Cron-based job scheduling |
| **HTTP Client** | Retry, exponential backoff |
| **Events** | Broadcast channel event bus |
| **Logging** | tracing with rolling files, trace ID |
| **Security** | CORS, CSRF, XSS protection, security headers |
| **CLI** | Project scaffolding, code generation (auto-register), hot reload |

## CLI

```bash
arcx new my-app           # Create project
arcx g c user             # Generate controller (auto-register)
arcx g s user             # Generate service (auto-register)
arcx g m auth             # Generate middleware (auto-register)
arcx g j cleanup          # Generate scheduled job (auto-register)
arcx dev                  # Dev server with hot reload
arcx info                 # Project stats
```

All generators auto-register into `mod.rs`, `router.rs`, or `main.rs` — zero manual wiring.

## Design Principles

- Convention over configuration — sensible defaults, minimal boilerplate
- One declaration, globally available — `services!{}` is the Rust-appropriate boundary
- Ctx is optional — handlers that don't need it simply don't declare it
- Middleware and Controller share the same Ctx — same API, same capabilities
- Your code, your rules — response format, auth strategy, middleware are all yours to define
- Zero runtime reflection — everything resolved at compile time

## License

MIT
