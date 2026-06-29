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

## Generated Controller Style

Controllers are pure async functions with free parameter signatures:

```rust
use arcx_core::prelude::*;
use crate::helper;

pub async fn index(_ctx: Context) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({ "items": [], "total": 0 })))
}

pub async fn show(_ctx: Context, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({ "id": id })))
}

pub async fn create(_ctx: Context, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(helper::created(json!({ "item": body })))
}
```

## Generated Router Style

Routes are declared freely — no forced RESTful conventions:

```rust
pub fn routes(r: &mut ArcxRouter) {
    r.get("/api/user", controller::user::index);
    r.get("/api/user/:id", controller::user::show);
    r.post("/api/user", controller::user::create);
    r.post("/api/user/login", controller::user::login);
    r.put("/api/user/:id/avatar", controller::user::upload_avatar);
}
```

## Links

- [GitHub](https://github.com/name-q/Arcx)
- [arcx-core](https://crates.io/crates/arcx-core) — Framework core library
