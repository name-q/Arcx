//! Health Controller

use arcx_core::prelude::*;

/// GET /api/health
pub async fn check(ctx: Context) -> AppResult<Json<Value>> {
    Ok(success(json!({
        "status": "ok",
        "app": ctx.config.app.name,
        "env": ctx.env(),
    })))
}
