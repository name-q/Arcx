# Arcx

[![Crates.io](https://img.shields.io/crates/v/arcx-core?label=arcx-core)](https://crates.io/crates/arcx-core)
[![Crates.io](https://img.shields.io/crates/v/arcx-cli?label=arcx-cli)](https://crates.io/crates/arcx-cli)
[![GitHub](https://img.shields.io/badge/GitHub-name--q%2FArcx-blue?logo=github)](https://github.com/name-q/Arcx)

A convention-over-configuration web framework for Rust, built on [Axum](https://github.com/tokio-rs/axum).

Arc(Architecture) + X(Extensible) — 约定优于配置，开箱即用。

## Features

- **Free-style routing** — `r.get/post/put/delete`, no forced conventions
- **Pure function handlers** — no traits, no macros, any parameter signature
- **Flexible responses** — return any `impl IntoResponse`, no forced format
- **Auth provider** — implement one trait, use any strategy (JWT/Session/OAuth)
- **Plugin system** — database, custom plugins with lifecycle management
- **Type-safe config** — multi-environment TOML config with hot reload
- **Built-in security** — CSRF, XSS protection, security headers, signed sessions
- **WebSocket** — trait-based WS handler with session management
- **Schedule** — cron-based job scheduling
- **HTTP Client** — reqwest wrapper with retry & exponential backoff
- **Event Bus** — broadcast channel driven event system
- **CLI Tool** — project scaffolding and code generation

## Quick Start

```bash
# Install CLI
cargo install arcx-cli

# Create project
arcx new my-app
cd my-app

# Run
cargo run

# Or with hot reload
arcx dev
```

## Project Structure

```
my-app/
├── src/
│   ├── main.rs              # Entry point
│   ├── router.rs            # Route declarations (free style)
│   ├── helper.rs            # Response format (your code, customizable)
│   ├── controller/          # Handler functions
│   ├── middleware/
│   │   └── auth.rs          # Auth implementation (your code)
│   ├── service/             # Business logic
│   └── model/               # Database entities
├── config/
│   ├── config.default.toml
│   └── config.prod.toml
└── Cargo.toml
```

## Example

```rust
// main.rs
use arcx_core::prelude::*;
use crate::middleware::auth::JwtAuth;

#[tokio::main]
async fn main() {
    Arcx::new()
        .auth(JwtAuth::new("secret"))  // optional
        .routes(router::routes)
        .run()
        .await;
}
```

```rust
// router.rs
pub fn routes(r: &mut ArcxRouter) {
    r.get("/api/home", controller::home::index);
    r.post("/api/home", controller::home::create);

    // Protected routes (requires .auth() in main.rs)
    r.guarded_scope("/api/admin", |s| {
        s.get("/profile", controller::admin::profile);
    });
}
```

```rust
// controller/home.rs
pub async fn index(ctx: Context) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({ "message": "Hello!" })))
}
```

```rust
// middleware/auth.rs — implement AuthProvider trait
#[async_trait]
impl AuthProvider for JwtAuth {
    async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError> {
        let token = parts.headers.get("Authorization") /* ... */;
        Ok(AuthUser { id: "user_1".into(), payload: json!({}) })
    }
}
```

## Middleware Philosophy

Every middleware has exactly two outcomes:
- **Pass** → `next()`, optionally inject data for the handler
- **Block** → return response directly (the handler never runs)

## CLI Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `arcx new <name>` | - | Create new project |
| `arcx generate controller <name>` | `arcx g c` | Generate controller |
| `arcx generate service <name>` | `arcx g s` | Generate service |
| `arcx generate model <name>` | `arcx g m` | Generate model |
| `arcx generate job <name>` | `arcx g j` | Generate job |
| `arcx dev [-p port]` | - | Dev server with hot reload |
| `arcx info` | - | Project stats |

## Architecture

```
crates/
├── arcx-core/     # Framework library
└── arcx-cli/      # CLI scaffolding tool
examples/
└── demo/          # Working example application
```

## License

MIT
