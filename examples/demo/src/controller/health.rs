//! Health Controller

use arcx_core::prelude::*;

/// GET /api/health — 直接返回 Json，不用 helper 也行
pub async fn check(ctx: Context) -> AppResult<impl IntoResponse> {
    Ok(Json(json!({
        "status": "ok",
        "app": ctx.config.app.name,
        "env": ctx.env(),
    })))
}
