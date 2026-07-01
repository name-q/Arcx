//! Home Controller

use arcx_core::prelude::*;
use crate::helper::response;

/// GET /api/home
pub async fn index(ctx: Context) -> AppResult<impl IntoResponse> {
    // 强类型访问
    let name = &ctx.config.app.name;
    let version = &ctx.config.app.version;

    // 动态访问（可读取 toml 中任意自定义字段）
    let port: Option<u16> = ctx.get("server.port");
    let custom: Option<String> = ctx.get("custom.greeting");

    Ok(response::success(json!({
        "name": name,
        "version": version,
        "port": port,
        "greeting": custom.unwrap_or_else(|| "Hello from Arcx!".into()),
    })))
}

/// GET /api/home/:id
pub async fn show(ctx: Context, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    let env = ctx.env();
    Ok(response::success(json!({ "id": id, "env": env })))
}

/// POST /api/home
pub async fn create(_ctx: Context, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(response::created(json!({ "item": body })))
}

/// PUT /api/home/:id
pub async fn update(_ctx: Context, Path(id): Path<u64>, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(response::success(json!({ "id": id, "updated": body })))
}

/// DELETE /api/home/:id
pub async fn destroy(_ctx: Context, Path(_id): Path<u64>) -> AppResult<impl IntoResponse> {
    Ok(response::no_content())
}
