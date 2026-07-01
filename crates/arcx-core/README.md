# arcx-core

The core library of **Arcx** — a convention-over-configuration web framework for Rust built on [Axum](https://github.com/tokio-rs/axum).

## Features

- **Free-style routing** — `r.get/post/put/delete` with any handler signature
- **Ctx — optional request-scoped service locator** — config, plugins, service discovery
- **Service trait** — `ctx.service::<T>()` lazy creation + per-request caching + inter-service calls
- **Zero-boilerplate controllers** — pure async functions, Ctx optional
- **Flexible responses** — return any `impl IntoResponse`, no forced format
- **Plugin system** — Database, JWT, custom plugins with resource injection
- **Auth provider** — implement `AuthProvider` trait, use any auth strategy
- **Auto middleware** — CORS, logging, security headers, configurable
- **Multi-env config** — TOML config with environment-based overrides, import support
- **Error handling** — `AppError` with proper HTTP status codes (400/401/404/422/500)
- **Route guards** — `guarded_scope` for authenticated routes
- **Validation** — `ValidJson<T>` with validator derive macros
- **And more** — WebSocket, Schedule jobs, EventBus, HttpClient, Session

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
        // .auth(MyAuth::new("secret"))  // optional: enable guarded_scope
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
    // Public routes
    r.get("/api/home", controller::home::index);
    r.get("/api/home/:id", controller::home::show);
    r.post("/api/home", controller::home::create);

    // Authenticated routes (requires .auth() in main.rs)
    r.guarded_scope("/api/admin", |s| {
        s.get("/profile", controller::admin::profile);
    });
}
```

### Ctx — Request-scoped service locator (optional)

```rust
use arcx_core::prelude::*;

// Don't need Ctx? Don't write it.
pub async fn destroy(Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    Ok(response::no_content())
}

// Need config/plugins/services? Add Ctx.
pub async fn show(ctx: Ctx, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    let env = ctx.env();
    let user = ctx.service::<UserService>().find_by_id(id).await?;
    Ok(Json(user))
}
```

### Service trait — Inter-service calls

```rust
use std::sync::Arc;
use arcx_core::prelude::*;

pub struct UserService {
    ctx: Ctx,
}

impl Service for UserService {
    fn create(ctx: &Ctx) -> Arc<Self> {
        Arc::new(Self { ctx: ctx.clone() })
    }
}

impl UserService {
    pub async fn find_with_orders(&self, id: i64) -> AppResult<Value> {
        let user = self.find_by_id(id).await?;
        // Inter-service call — just like ctx.service.order in other frameworks
        let orders = self.ctx.service::<OrderService>().find_by_user(id).await?;
        Ok(json!({ "user": user, "orders": orders }))
    }
}
```

### Auth — Implement your own strategy

```rust
use arcx_core::prelude::*;

pub struct JwtAuth { secret: String }

#[async_trait]
impl AuthProvider for JwtAuth {
    async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError> {
        let token = parts.headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::unauthorized("Missing token"))?;

        let claims = verify_jwt(token, &self.secret)?;

        Ok(AuthUser {
            id: claims.sub,
            payload: json!({ "role": claims.role }),
        })
    }
}
```

### helper/ — Your utilities (customizable)

The `src/helper/` directory is **your code** — the framework doesn't depend on it. Put response formatters, crypto helpers, date utils, or anything you need:

```
src/helper/
├── mod.rs        # re-exports
├── response.rs   # response format helpers
├── crypto.rs     # your crypto utils
└── time.rs       # your date/time utils
```

```rust
// src/helper/response.rs
pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
    Json(json!({ "code": 0, "data": data, "message": "success" }))
}
```

### Error handling

```rust
return Err(AppError::not_found("User not found"));
return Err(AppError::unauthorized("Login required"));
return Err(AppError::validation(vec![
    FieldError { field: "title".into(), message: "required".into(), code: "missing_field".into() }
]));
```

## Project Structure

```
src/
├── main.rs           # Entry point
├── router.rs         # Route declarations (free style)
├── helper/           # Response helpers & utilities (your code)
├── controller/       # Handler functions (Ctx optional)
├── service/          # Business logic (Service trait)
├── middleware/       # Auth & custom middleware
└── model/            # Database entities
config/
├── config.default.toml
└── config.prod.toml
```

## Links

- [GitHub](https://github.com/name-q/Arcx)
- [arcx-cli](https://crates.io/crates/arcx-cli) — CLI scaffolding tool
