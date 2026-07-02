# arcx-cli

CLI scaffolding tool for **Arcx** — a convention-over-configuration web framework for Rust.

## Install

```bash
cargo install arcx-cli
```

## Commands

### Create a new project

```bash
arcx new myapp
cd myapp
cargo run
```

Generated structure:

```
myapp/
├── src/
│   ├── main.rs           # Entry point
│   ├── router.rs         # Route declarations (free style)
│   ├── prelude.rs        # Project-level prelude (one import everywhere)
│   ├── controller/
│   │   └── home.rs       # Example controller
│   ├── middleware/
│   │   └── auth.rs       # Auth implementation (customizable)
│   ├── service/
│   │   ├── mod.rs        # services! container
│   │   └── user.rs       # Example service
│   ├── schedule/         # Cron jobs (when generated)
│   └── helper/
│       └── response.rs   # Response format helpers
├── config/
│   ├── config.default.toml
│   └── config.prod.toml
└── Cargo.toml
```

### Generate code (auto-register)

All generators automatically register into the appropriate files — zero manual wiring.

```bash
# Generate controller (alias: g c)
arcx g c user
# → Creates src/controller/user.rs
# → Auto-registers in controller/mod.rs + router.rs

# Generate service (alias: g s)
arcx g s order
# → Creates src/service/order.rs
# → Auto-registers in service/mod.rs (services! macro)

# Generate middleware (alias: g m)
arcx g m rate_limit
# → Creates src/middleware/rate_limit.rs
# → Auto-registers in middleware/mod.rs

# Generate schedule job (alias: g j)
arcx g j cleanup
# → Creates src/schedule/cleanup.rs
# → Auto-registers in schedule/mod.rs + main.rs (.schedule())
```

### Development server (hot reload)

```bash
arcx dev
arcx dev -p 8080  # custom port
```

Watches file changes, recompiles, and restarts automatically.

### Project info

```bash
arcx info
```

## Configuration

Each middleware/feature manages its own config section:

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
```

Rule: **has section + `enable = true` = on.** No section or `enable = false` = off.

## Auth System

Projects include a `middleware/auth.rs` implementing `AuthProvider` trait.
Enable it in `main.rs`:

```rust
use crate::middleware::auth::JwtAuth;

Arcx::new()
    .auth(JwtAuth::new("your-secret"))
    .routes(router::routes)
    .run()
    .await;
```

Then use `guarded_scope` in your router:

```rust
r.guarded_scope("/api/admin", |s| {
    s.get("/profile", controller::admin::profile);
});
```

The `AuthProvider` trait lets you implement any auth strategy (JWT, Session, OAuth, etc).

## Links

- [GitHub](https://github.com/name-q/Arcx)
- [arcx-core](https://crates.io/crates/arcx-core) — Framework core library
- [arcx-macros](https://crates.io/crates/arcx-macros) — Proc macros
