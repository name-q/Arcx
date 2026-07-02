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
    r.get("/api/home", controller::home::index);
    r.post("/api/home", controller::home::create);
    r.put("/api/home/:id", controller::home::update);
    r.delete("/api/home/:id", controller::home::destroy);

    // Protected routes (requires .auth() in main.rs)
    r.guarded_scope("/api/admin", |s| {
        s.get("/profile", controller::admin::profile);
    });
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

Multi-environment TOML with dot-notation access:

```rust
let port: Option<u16> = ctx.get("server.port");
let redis_url: Option<String> = ctx.get("redis.url");
```

Config files support `import` for splitting:
```toml
import = ["config/custom/redis.toml"]

[app]
name = "my-app"
env = "dev"
```

### Plugins

Database, caching, and custom plugins — register once, access via `ctx.plugin::<T>()`:

```toml
# config.default.toml
[database]
url = "postgres://localhost/mydb"
```

```rust
let db = ctx.plugin::<DatabasePlugin>()?;
```

## Features

| Category | Capabilities |
|----------|-------------|
| **Routing** | Free-style, guarded scopes, path params |
| **Services** | `#[service]` macro, container pattern, inter-service calls |
| **Auth** | Pluggable AuthProvider trait, route guards |
| **Config** | Multi-env TOML, dot-notation, import, hot reload |
| **Plugins** | Database (SeaORM), JWT, custom plugin trait |
| **Middleware** | CORS, request logger, security headers, CSRF |
| **Session** | HMAC-signed cookies, extensible store |
| **WebSocket** | Trait-based handler, session management |
| **Schedule** | Cron-based job scheduling |
| **HTTP Client** | Retry, exponential backoff |
| **Events** | Broadcast channel event bus |
| **Logging** | tracing with rolling files, trace ID |
| **CLI** | Project scaffolding, code generation, hot reload dev server |

## CLI

```bash
arcx new my-app           # Create project
arcx g c user             # Generate controller
arcx g s user             # Generate service
arcx g m user             # Generate model
arcx g j cleanup          # Generate scheduled job
arcx dev                  # Dev server with hot reload
arcx info                 # Project stats
```

## Middleware Philosophy

Every middleware has exactly two outcomes:
- **Pass** → call next, optionally inject data
- **Block** → return response directly

## Design Principles

- Convention over configuration — sensible defaults, minimal boilerplate
- One declaration, globally available — `services!{}` is the Rust-appropriate boundary
- Ctx is optional — handlers that don't need it simply don't declare it
- Your code, your rules — response format, auth strategy, middleware are all yours to define
- Zero runtime reflection — everything resolved at compile time

## License

MIT
