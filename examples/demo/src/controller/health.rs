use crate::prelude::*;

/// GET /api/health
pub async fn check(ctx: Ctx) -> AppResult<impl IntoResponse> {
    Ok(Json(json!({
        "status": "ok",
        "app": ctx.config().app.name,
        "env": ctx.env(),
    })))
}
