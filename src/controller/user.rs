use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};

use crate::service::user::UserService;

/// GET /api/user/:id
pub async fn info(Path(id): Path<u64>) -> Json<Value> {
    match UserService::find_by_id(id).await {
        Some(user) => Json(json!({
            "success": true,
            "data": user
        })),
        None => Json(json!({
            "success": false,
            "message": "用户不存在"
        })),
    }
}

/// GET /api/user
pub async fn list() -> Json<Value> {
    let users = UserService::find_all().await;
    Json(json!({
        "success": true,
        "data": users
    }))
}
