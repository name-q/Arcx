# arcx-core

The core library of **Arcx** — a convention-over-configuration web framework for Rust built on [Axum](https://github.com/tokio-rs/axum).

## Features

- **Free-style routing** — `r.get/post/put/delete` with any handler signature
- **Zero-boilerplate controllers** — pure async functions, no traits needed
- **Flexible responses** — return any `impl IntoResponse`, no forced format
- **Plugin system** — Database, JWT, custom plugins with resource injection
- **Auto middleware** — CORS, logging, security headers, configurable
- **Multi-env config** — TOML config with environment-based overrides
- **Error handling** — `AppError` with proper HTTP status codes (400/401/404/422/500)
- **Route guards** — `guarded_scope` for authenticated routes
- **Validation** — `ValidJson<T>` with validator derive macros
- **And more** — WebSocket, Schedule jobs, EventBus, HttpClient, Session

## Quick Start

```rust
use arcx_core::prelude::*;

mod controller;
mod helper;
mod router;

#[tokio::main]
async fn main() {
    Arcx::new()
        .routes(router::routes)
        .run()
        .await;
}
```

### router.rs — Free-style routing

```rust
use arcx_core::prelude::*;
use crate::controller;

pub fn routes(r: &mut ArcxRouter) {
    r.get("/api/home", controller::home::index);
    r.get("/api/home/:id", controller::home::show);
    r.post("/api/home", controller::home::create);
    r.put("/api/home/:id", controller::home::update);
    r.delete("/api/home/:id", controller::home::destroy);

    // Authenticated routes
    r.guarded_scope("/api/admin", |s| {
        s.get("/dashboard", controller::admin::dashboard);
    });
}
```

### controller — Pure functions, any params

```rust
use arcx_core::prelude::*;
use crate::helper;

pub async fn index(ctx: Context) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({ "message": "Hello!" })))
}

pub async fn show(_ctx: Context, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({ "id": id })))
}

pub async fn create(_ctx: Context, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(helper::created(json!({ "item": body })))
}
```

### helper.rs — Your response format (customizable)

```rust
pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
    Json(json!({ "code": 0, "data": data, "message": "success" }))
}

pub fn created<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "code": 0, "data": data, "message": "created" })))
}

pub fn no_content() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
```

The `helper.rs` is **your code** — modify it freely. The framework doesn't depend on it.

### Error handling

```rust
// Throw errors anywhere in controller/service
return Err(AppError::not_found("User not found"));
return Err(AppError::bad_request("Invalid parameter"));
return Err(AppError::unauthorized("Login required"));
return Err(AppError::validation(vec![
    FieldError { field: "title".into(), message: "required".into(), code: "missing_field".into() }
]));
```

## Links

- [GitHub](https://github.com/name-q/Arcx)
- [arcx-cli](https://crates.io/crates/arcx-cli) — CLI scaffolding tool
