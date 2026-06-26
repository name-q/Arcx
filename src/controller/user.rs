//! 用户 Controller（框架示例）
//! 路由前缀: /api/user
//! 演示：路由定义、Context 使用、参数校验、路由守卫

use axum::extract::Path;
use axum::{routing::{get, post}, Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::context::{AppState, Context};
use crate::error::{AppError, AppResult, success};
use crate::extract::ValidJson;
use crate::guard::CurrentUser;

/// 注册路由（公开路由）
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/login", post(login))
}

/// 受保护路由 —— 需要鉴权守卫
/// 框架约定：如果 controller 暴露 protected_routes()，自动添加 AuthGuard
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/profile", get(profile))
        .route("/create", post(create))
        .route("/:id", get(info))
}

/// GET /api/user — 公开，返回用户列表
async fn list(_ctx: Context) -> AppResult<Json<serde_json::Value>> {
    Ok(success(serde_json::json!([
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"},
    ])))
}

/// POST /api/user/login — 公开，演示登录获取 token
async fn login(ctx: Context) -> AppResult<Json<serde_json::Value>> {
    use crate::plugin::builtin::jwt::JwtService;

    let jwt = ctx.resource::<JwtService>().ok_or_else(|| {
        AppError::Internal("JWT plugin not enabled".to_string())
    })?;

    // 示例：直接签发（实际应验证密码）
    let token = jwt.sign("user_1", Some(serde_json::json!({"role": "admin"})))
        .map_err(|e| AppError::Internal(e))?;

    Ok(success(serde_json::json!({
        "token": token,
        "expire": jwt.expire_seconds(),
    })))
}

/// GET /api/user/profile — 需要登录
async fn profile(user: CurrentUser) -> AppResult<Json<serde_json::Value>> {
    Ok(success(serde_json::json!({
        "sub": user.sub,
        "claims": {
            "exp": user.claims.exp,
            "iat": user.claims.iat,
            "data": user.claims.data,
        }
    })))
}

/// GET /api/user/:id — 需要登录
async fn info(_user: CurrentUser, Path(id): Path<u64>) -> AppResult<Json<serde_json::Value>> {
    if id > 100 {
        return Err(AppError::NotFound(format!("用户 {} 不存在", id)));
    }
    Ok(success(serde_json::json!({"id": id, "name": "Demo User"})))
}

/// 创建用户请求体
#[derive(Deserialize, Validate)]
struct CreateUserDto {
    #[validate(length(min = 1, message = "名称不能为空"))]
    name: String,

    #[validate(email(message = "邮箱格式不正确"))]
    email: String,
}

/// POST /api/user/create — 需要登录
async fn create(
    _user: CurrentUser,
    body: ValidJson<CreateUserDto>,
) -> AppResult<Json<serde_json::Value>> {
    let dto = body.into_inner();
    Ok(success(serde_json::json!({
        "id": 1,
        "name": dto.name,
        "email": dto.email,
    })))
}
