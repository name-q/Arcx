# arcx-core

The core framework library for [Arcx](https://github.com/name-q/Arcx) — a convention-over-configuration web framework for Rust with AI orchestration capabilities.

## Features

- **Convention over configuration** — sensible defaults, override only what you need
- **Minimal boilerplate** — 3-line `main.rs`, centralized route declarations
- **RESTful resources** — declare a resource, get CRUD routes automatically
- **Plugin system** — database, JWT, and custom plugins with dependency injection
- **Built-in security** — CORS, CSRF, security headers, auth guards
- **Full observability** — structured logging, trace IDs, request logging
- **Developer experience** — config hot-reload, event bus, graceful shutdown

## Quick Start

```rust
// main.rs
mod controller;
mod router;

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
// router.rs
use arcx_core::prelude::*;
use crate::controller;

pub fn routes(r: &mut ArcxRouter) {
    r.resources("/api/user", controller::user::handlers());
    r.get("/api/health", controller::health::check);
}
```

```rust
// controller/user.rs
use arcx_core::prelude::*;

pub async fn index(_ctx: Context) -> AppResult<Json<Value>> {
    Ok(success(json!({ "users": [] })))
}

pub async fn show(_ctx: Context, Path(id): Path<u64>) -> AppResult<Json<Value>> {
    Ok(success(json!({ "id": id })))
}

pub fn handlers() -> ResourceHandlers {
    ResourceHandlers::new()
        .index(index)
        .show(show)
}
```

## Route Conventions

`r.resources("/api/user", handlers)` maps:

| Handler | HTTP Method | Path |
|---------|-------------|------|
| `index` | GET | `/api/user` |
| `show` | GET | `/api/user/:id` |
| `create` | POST | `/api/user` |
| `update` | PUT | `/api/user/:id` |
| `destroy` | DELETE | `/api/user/:id` |

Only registered handlers get routes. No handler = no route. No warnings.

## Configuration

```toml
# config/config.default.toml
[app]
name = "my-app"
version = "0.1.0"
env = "dev"

[server]
host = "127.0.0.1"
port = 3000

[middleware]
cors = true
logger = true
security = true

[plugin.database]
enable = true
url = "sqlite:./data.db?mode=rwc"

[plugin.jwt]
enable = true
secret = "your-secret-key"
expire = 86400
```

## Architecture

```
Request → Middleware (CORS/Logger/Security)
        → Router (ArcxRouter)
        → Guard (optional auth)
        → Controller (handler fn)
        → Service (business logic)
        → Plugin resources (DB, cache, etc.)
```

## Plugin System

```rust
use arcx_core::prelude::*;

// Access plugin resources in controllers
pub async fn index(ctx: Context) -> AppResult<Json<Value>> {
    let db = ctx.resource::<DatabaseConnection>().unwrap();
    // ...
}
```

## License

MIT
