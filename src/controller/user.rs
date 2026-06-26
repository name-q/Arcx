//! 用户 Controller（示例）
//! 路由前缀: /api/user
//! 演示框架的 Controller 写法约定

use axum::extract::Path;
use axum::{routing::get, Json, Router};
use serde::Deserialize;

use crate::context::{AppState, Context};
use crate::error::{AppError, AppResult, success};

/// 注册路由
/// 约定：每个 controller 暴露此函数，定义该模块下的所有路由
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(info))
}

/// GET /api/user
/// 获取用户列表
async fn list(_ctx: Context) -> AppResult<Json<serde_json::Value>> {
    // 示例数据（实际项目中调用 Service 层）
    let users = vec![
        serde_json::json!({"id": 1, "name": "Alice"}),
        serde_json::json!({"id": 2, "name": "Bob"}),
    ];
    Ok(success(users))
}

/// GET /api/user/:id
/// 获取单个用户
async fn info(_ctx: Context, Path(id): Path<u64>) -> AppResult<Json<serde_json::Value>> {
    if id > 100 {
        return Err(AppError::NotFound(format!("用户 {} 不存在", id)));
    }
    Ok(success(serde_json::json!({"id": id, "name": "Demo User"})))
}

/// POST /api/user
/// 创建用户
#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: Option<String>,
}

async fn create(
    _ctx: Context,
    Json(body): Json<CreateUserRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if body.name.is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    Ok(success(serde_json::json!({
        "id": 1,
        "name": body.name,
        "email": body.email,
    })))
}
