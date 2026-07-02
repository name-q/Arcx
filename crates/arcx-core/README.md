# arcx-core

The core library of **Arcx** — a convention-over-configuration web framework for Rust built on [Axum](https://github.com/tokio-rs/axum).

## Features

- **Free-style routing** — `r.get/post/put/delete` with global/scope/route-level middleware
- **Ctx** — optional request-scoped service locator with quick access methods
- **Services** — `#[service]` macro + `services!{}` container, inter-service calls
- **Middleware** — shares same Ctx with controllers, onion model, `ctx.set()`/`ctx.get::<T>()`
- **Auth** — implement `AuthProvider` trait, use any strategy (JWT/Session/OAuth)
- **Config** — multi-env TOML, dot-notation `ctx.conf("key")`, import support, hot reload
- **Plugins** — Database, JWT, custom plugins with resource injection
- **Schedule** — cron-based job scheduling via `ScheduleJob` trait
- **Security** — CORS, CSRF, XSS protection, security headers (each independently configurable)
- **And more** — WebSocket, EventBus, HttpClient, Session, tracing

## Quick Start

```rust
use arcx_core::prelude::*;

mod controller;
mod helper;
mod middleware;
mod router;

#[tokio::main]
async fn main() {
    Arcx::new()
        .routes(router::routes)
        .run()
        .await;
}
```

### Routing — three levels of middleware

```rust
pub fn routes(r: &mut ArcxRouter) {
    // Global
    r.middleware(middleware::log::handle);

    // Standard
    r.get("/api/home", controller::home::index);

    // Route-level (chainable)
    r.get("/api/admin", controller::admin::dashboard)
        .middleware(middleware::auth::handle);

    // Scope-level
    r.scope("/api/v2", |s| {
        s.middleware(middleware::auth::handle);
        s.get("/users", controller::user::list);
    });

    // Guarded scope (requires .auth() in main.rs)
    r.guarded_scope("/api/admin", |s| {
        s.get("/profile", controller::admin::profile);
    });
}
```

### Middleware — same Ctx as controllers

```rust
use arcx_core::prelude::*;

pub async fn handle(ctx: Ctx, next: Next, parts: ReqParts) -> Response {
    let env = ctx.env();
    ctx.set(MyData { role: "admin".into() });
    ctx.next(next, parts).await
}
```

### Ctx — request-scoped service locator (optional)

```rust
pub async fn show(ctx: Ctx, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    let host = ctx.header("host");
    let ip = ctx.ip();
    let params: MyQuery = ctx.query()?;
    let port: Option<u16> = ctx.conf("server.port");
    let user = ctx.services().user.find_by_id(id).await?;
    Ok(Json(user))
}

// Don't need Ctx? Don't write it.
pub async fn health() -> &'static str { "ok" }
```

### Services — `#[service]` macro

```rust
use crate::prelude::*;

#[service]
impl UserService {
    pub async fn find_by_id(&self, id: u64) -> AppResult<Value> {
        Ok(json!({ "id": id, "name": format!("User_{}", id) }))
    }

    pub async fn find_with_orders(&self, id: u64) -> AppResult<Value> {
        let user = self.find_by_id(id).await?;
        let orders = self.ctx.service::<OrderService>().find_by_user(id).await?;
        Ok(json!({ "user": user, "orders": orders }))
    }
}
```

### Configuration — each feature self-contained

```toml
[server]
port = 3000

[cors]
enable = true
allowed_origins = ["*"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]

[request_logger]
enable = true

[security]
enable = true
csrf = false
```

### Auth — pluggable

```rust
pub struct JwtAuth { secret: String }

#[async_trait]
impl AuthProvider for JwtAuth {
    async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError> {
        // your logic
        Ok(AuthUser { id: "user_1".into(), payload: json!({}) })
    }
}
```

```rust
Arcx::new()
    .auth(JwtAuth::new("secret"))
    .routes(router::routes)
    .run()
    .await;
```

### Schedule jobs

```rust
pub struct CleanupJob;

impl ScheduleJob for CleanupJob {
    fn name(&self) -> &str { "cleanup" }
    fn cron(&self) -> &str { "0 */5 * * * *" }
    fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async { println!("[CleanupJob] executing"); })
    }
}
```

```rust
Arcx::new()
    .routes(router::routes)
    .schedule(CleanupJob)
    .run()
    .await;
```

## Links

- [GitHub](https://github.com/name-q/Arcx)
- [arcx-cli](https://crates.io/crates/arcx-cli) — CLI scaffolding tool
- [arcx-macros](https://crates.io/crates/arcx-macros) — Proc macros (`#[service]`)
