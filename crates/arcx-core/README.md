# arcx-core

The core library of **Arcx** — a convention-over-configuration web framework for Rust built on [Axum](https://github.com/tokio-rs/axum).

## Features

- **Free-style routing** — `r.get/post/put/delete` with any handler signature
- **Zero-boilerplate controllers** — pure async functions, no traits needed
- **Flexible responses** — return any `impl IntoResponse`, no forced format
- **Plugin system** — Database, JWT, custom plugins with resource injection
- **Auth provider** — implement `AuthProvider` trait, use any auth strategy
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
mod middleware;
mod router;

use crate::middleware::auth::JwtAuth;

#[tokio::main]
async fn main() {
    Arcx::new()
        .auth(JwtAuth::new("your-secret"))  // optional: enable guarded_scope
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
        s.get("/dashboard", controller::admin::dashboard);
    });
}
```

### Auth — Implement your own strategy

```rust
use arcx_core::prelude::*;

pub struct JwtAuth { secret: String }

#[async_trait]
impl AuthProvider for JwtAuth {
    async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError> {
        // 1. Extract token from header/cookie/query — your choice
        let token = parts.headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::unauthorized("Missing token"))?;

        // 2. Verify token — JWT, session, OAuth, whatever you want
        let claims = verify_jwt(token, &self.secret)?;

        // 3. Return AuthUser — handler can extract it
        Ok(AuthUser {
            id: claims.sub,
            payload: json!({ "role": claims.role }),
        })
    }
}
```

### Middleware behavior

Every middleware (including auth guard) has exactly two outcomes:

- **Pass** — call `next`, optionally inject data for the handler
- **Block** — return a response directly (error or otherwise)

```
Request → guarded_scope middleware
          ↓
    AuthProvider::authenticate(request_parts)
          ↓
    ┌─ Ok(AuthUser) → inject into request → handler executes
    └─ Err(AppError) → return error response, handler never runs
```

### controller — Pure functions

```rust
use arcx_core::prelude::*;
use crate::helper;

// Public handler
pub async fn index(ctx: Context) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({ "message": "Hello!" })))
}

// Protected handler — AuthUser extracted automatically
pub async fn profile(_ctx: Context, user: AuthUser) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({ "id": user.id, "role": user.payload["role"] })))
}
```

### helper.rs — Your response format (customizable)

```rust
pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
    Json(json!({ "code": 0, "data": data, "message": "success" }))
}

pub fn created<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "code": 0, "data": data })))
}

pub fn no_content() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
```

The `helper.rs` is **your code** — modify it freely. The framework doesn't depend on it.

### Error handling

```rust
return Err(AppError::not_found("User not found"));
return Err(AppError::unauthorized("Login required"));
return Err(AppError::validation(vec![
    FieldError { field: "title".into(), message: "required".into(), code: "missing_field".into() }
]));
```

## Links

- [GitHub](https://github.com/name-q/Arcx)
- [arcx-cli](https://crates.io/crates/arcx-cli) — CLI scaffolding tool
