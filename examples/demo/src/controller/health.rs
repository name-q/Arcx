//! Health Controller

use arcx_core::prelude::*;

/// GET /api/health — 直接返回 Json，不需要 Ctx 也行
pub async fn check(ctx: Ctx) -> AppResult<impl IntoResponse> {
    Ok(Json(json!({
        "status": "ok",
        "app": ctx.config().app.name,
        "env": ctx.env(),
    })))
}
