//! Home Controller
//! Routes: /api/home

use arcx_core::prelude::*;

/// GET /api/home
pub async fn index(ctx: Context) -> AppResult<Json<Value>> {
    Ok(success(json!({
        "name": ctx.config.app.name,
        "version": ctx.config.app.version,
        "message": "Welcome to Arcx!"
    })))
}

/// GET /api/home/:id
pub async fn show(_ctx: Context, Path(id): Path<u64>) -> AppResult<Json<Value>> {
    Ok(success(json!({ "id": id })))
}

/// 导出 handlers
pub fn handlers() -> ResourceHandlers {
    ResourceHandlers::new()
        .index(index)
        .show(show)
}
