use axum::extract::Path;
use axum::Json;
use serde_json::Value;

use crate::error::{AppError, AppResult, success};
use crate::service::user::UserService;

/// GET /api/user/:id
pub async fn info(Path(id): Path<u64>) -> AppResult<Json<Value>> {
    let user = UserService::find_by_id(id)
        .await
        .ok_or(AppError::NotFound(format!("用户 {} 不存在", id)))?;

    Ok(success(user))
}

/// GET /api/user
pub async fn list() -> AppResult<Json<Value>> {
    let users = UserService::find_all().await;
    Ok(success(users))
}
