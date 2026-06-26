use axum::extract::Path;
use axum::Json;
use serde_json::Value;

use crate::context::Context;
use crate::error::{AppError, AppResult, success};
use crate::service::user::{CreateUserDto, UserService};

/// GET /api/user/:id
pub async fn info(ctx: Context, Path(id): Path<u64>) -> AppResult<Json<Value>> {
    tracing::debug!("app debug mode: {}", ctx.config.app.debug);

    let user = UserService::find_by_id(id)
        .await
        .ok_or(AppError::NotFound(format!("用户 {} 不存在", id)))?;

    Ok(success(user))
}

/// GET /api/user
pub async fn list(_ctx: Context) -> AppResult<Json<Value>> {
    let users = UserService::find_all().await;
    Ok(success(users))
}

/// POST /api/user
/// Body: { "name": "xxx", "email": "xxx@xxx.com" }
pub async fn create(_ctx: Context, Json(dto): Json<CreateUserDto>) -> AppResult<Json<Value>> {
    // 参数校验
    if dto.name.is_empty() {
        return Err(AppError::BadRequest("name 不能为空".to_string()));
    }
    if !dto.email.contains('@') {
        return Err(AppError::BadRequest("email 格式不正确".to_string()));
    }

    let user = UserService::create(dto).await;
    Ok(success(user))
}
