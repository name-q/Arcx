//! Home Controller

use arcx_core::prelude::*;
use crate::helper;

/// GET /api/home
pub async fn index(ctx: Context) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({
        "name": ctx.config.app.name,
        "version": ctx.config.app.version,
        "message": "Welcome to Arcx!"
    })))
}

/// GET /api/home/:id
pub async fn show(_ctx: Context, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({ "id": id })))
}

/// POST /api/home
pub async fn create(_ctx: Context, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(helper::created(json!({ "item": body })))
}

/// PUT /api/home/:id
pub async fn update(_ctx: Context, Path(id): Path<u64>, Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({ "id": id, "updated": body })))
}

/// DELETE /api/home/:id
pub async fn destroy(_ctx: Context, Path(_id): Path<u64>) -> AppResult<impl IntoResponse> {
    Ok(helper::no_content())
}
