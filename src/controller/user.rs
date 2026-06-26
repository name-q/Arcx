//! 用户 Controller（框架示例）
//! 路由前缀: /api/user
//! 演示：路由定义、Context 使用、参数校验

use axum::extract::Path;
use axum::{routing::get, Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::context::{AppState, Context};
use crate::error::{AppError, AppResult, success};
use crate::extract::ValidJson;

/// 注册路由
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(info))
}

/// GET /api/user
async fn list(_ctx: Context) -> AppResult<Json<serde_json::Value>> {
    let users = vec![
        serde_json::json!({"id": 1, "name": "Alice"}),
        serde_json::json!({"id": 2, "name": "Bob"}),
    ];
    Ok(success(users))
}

/// GET /api/user/:id
async fn info(_ctx: Context, Path(id): Path<u64>) -> AppResult<Json<serde_json::Value>> {
    if id > 100 {
        return Err(AppError::NotFound(format!("用户 {} 不存在", id)));
    }
    Ok(success(serde_json::json!({"id": id, "name": "Demo User"})))
}

/// 创建用户请求体 —— 使用 validator 声明式校验
#[derive(Deserialize, Validate)]
struct CreateUserDto {
    #[validate(length(min = 1, message = "名称不能为空"))]
    name: String,

    #[validate(email(message = "邮箱格式不正确"))]
    email: String,
}

/// POST /api/user
/// 演示 ValidJson 自动校验
async fn create(
    _ctx: Context,
    body: ValidJson<CreateUserDto>,
) -> AppResult<Json<serde_json::Value>> {
    let dto = body.into_inner();
    Ok(success(serde_json::json!({
        "id": 1,
        "name": dto.name,
        "email": dto.email,
    })))
}
