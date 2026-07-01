//! Home Controller — 展示 Ctx 可选使用

use arcx_core::prelude::*;
use crate::helper::response;

/// GET /api/home — 需要读配置，使用 Ctx
pub async fn index(ctx: Ctx) -> AppResult<impl IntoResponse> {
    let name = &ctx.config().app.name;
    let version = &ctx.config().app.version;

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

/// GET /api/home/:id — 需要 Ctx
pub async fn show(ctx: Ctx, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    let env = ctx.env();
    Ok(response::success(json!({ "id": id, "env": env })))
}

/// POST /api/home — 不需要 Ctx，直接不写
pub async fn create(Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(response::created(json!({ "item": body })))
}

/// PUT /api/home/:id — 不需要 Ctx
pub async fn update(Path(id): Path<u64>, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(response::success(json!({ "id": id, "updated": body })))
}

/// DELETE /api/home/:id — 不需要 Ctx
pub async fn destroy(Path(_id): Path<u64>) -> AppResult<impl IntoResponse> {
    Ok(response::no_content())
}
