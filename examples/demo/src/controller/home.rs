//! Home Controller — 展示 ctx.services() 链式调用

use arcx_core::prelude::*;
use crate::helper::response;
use crate::service::ServiceAccess;

/// GET /api/home — 读配置
pub async fn index(ctx: Ctx) -> AppResult<impl IntoResponse> {
    let name = &ctx.config().app.name;
    let version = &ctx.config().app.version;
    let port: Option<u16> = ctx.get("server.port");
    let custom: Option<String> = ctx.get("custom.greeting");

    Ok(response::success(json!({
        "name": name,
        "version": version,
        "port": port,
        "greeting": custom.unwrap_or_else(|| "Hello from Arcx!".into()),
    })))
}

/// GET /api/home/:id — ctx.services().user 链式调用
pub async fn show(ctx: Ctx, Path(id): Path<String>) -> AppResult<impl IntoResponse> {
    let user = ctx.services().user.get_profile(&id).await?;
    Ok(response::success(user))
}

/// GET /api/home/:id/detail — Service 互调（user 内部调 order）
pub async fn detail(ctx: Ctx, Path(id): Path<String>) -> AppResult<impl IntoResponse> {
    let data = ctx.services().user.get_user_with_orders(&id).await?;
    Ok(response::success(data))
}

/// POST /api/home — 不需要 Ctx，纯函数
pub async fn create(Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(response::created(json!({ "item": body })))
}

/// DELETE /api/home/:id — 不需要 Ctx
pub async fn destroy(Path(_id): Path<u64>) -> AppResult<impl IntoResponse> {
    Ok(response::no_content())
}
