# Arcx

A convention-over-configuration web framework for Rust, built on [Axum](https://github.com/tokio-rs/axum).

Arc(Architecture) + X(Extensible) — 约定优于配置，开箱即用。

## Features

- **Convention over Configuration** — file-based routing, auto controller loading
- **Plugin System** — database, JWT, custom plugins with lifecycle management
- **Type-safe Config** — multi-environment TOML config with hot reload
- **Built-in Security** — CSRF, XSS protection, security headers, signed sessions
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
│   ├── controller/          # Route handlers (file = route prefix)
│   │   ├── mod.rs
│   │   └── home.rs          # → /api/home
│   ├── service/             # Business logic
│   └── model/               # Database entities
├── config/
│   ├── config.default.toml  # Base config
│   └── config.prod.toml     # Production overrides
└── Cargo.toml
```

## Example Controller

```rust
use axum::{routing::get, Json, Router};
use arcx_core::prelude::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/:id", get(detail))
}

async fn index(ctx: Context) -> AppResult<Json<serde_json::Value>> {
    Ok(success(json!({ "message": "hello" })))
}

async fn detail(
    ctx: Context,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(success(json!({ "id": id })))
}
```

## Configuration

```toml
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
├── arcx-core/     # Framework library (the engine)
└── arcx-cli/      # CLI scaffolding tool
examples/
└── demo/          # Working example application
```

## License

MIT
