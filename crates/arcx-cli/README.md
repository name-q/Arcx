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
│   ├── helper.rs         # Response helpers (customizable)
│   ├── controller/
│   │   └── home.rs       # Example controller
│   ├── middleware/
│   │   └── auth.rs       # Auth implementation (customizable)
│   └── service/
├── config/
│   ├── config.default.toml
│   └── config.prod.toml
└── Cargo.toml
```

### Generate code

```bash
# Generate controller (alias: g c)
arcx g c user
# → Creates src/controller/user.rs
# → Auto-registers in mod.rs and router.rs

# Generate service (alias: g s)
arcx g s user

# Generate model (alias: g m)
arcx g m user

# Generate schedule job (alias: g j)
arcx g j cleanup
```

### Development server (hot reload)

```bash
arcx dev
arcx dev -p 8080  # custom port
```

### Project info

```bash
arcx info
```

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
