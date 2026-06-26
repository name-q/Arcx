//! 鉴权守卫
//! 
//! 从请求头提取 Authorization: Bearer <token>，验证 JWT。
//! 验证通过后将用户信息注入到请求扩展中，后续 handler 可通过 CurrentUser 提取。

use axum::{
    extract::{Request, State},
    http::{self, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::context::AppState;
use crate::plugin::builtin::jwt::{Claims, JwtService};

/// 当前登录用户 —— 从已验证的 token 中提取
/// 
/// 用法：
/// ```rust
/// async fn profile(user: CurrentUser) -> impl IntoResponse {
///     format!("Hello, {}", user.sub)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub sub: String,
    pub claims: Claims,
}

/// 实现 FromRequestParts，让 handler 可以直接提取 CurrentUser
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "success": false,
                        "error": { "code": 401, "message": "未登录" }
                    })),
                )
                    .into_response()
            })
    }
}

/// 鉴权守卫中间件
/// 
/// 使用方式：
/// ```rust
/// .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth_guard))
/// ```
pub async fn auth_guard(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    // 从 header 提取 token
    let token = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": { "code": 401, "message": "缺少 Authorization header" }
                })),
            )
                .into_response();
        }
    };

    // 获取 JWT 服务并验证
    let jwt_service = match state.resource::<JwtService>() {
        Some(s) => s,
        None => {
            tracing::error!("AuthGuard: JWT plugin not configured");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": { "code": 500, "message": "服务器配置错误" }
                })),
            )
                .into_response();
        }
    };

    // 验证 token
    match jwt_service.verify(token) {
        Ok(claims) => {
            let current_user = CurrentUser {
                sub: claims.sub.clone(),
                claims,
            };
            // 注入到请求扩展，后续 handler 可提取
            req.extensions_mut().insert(current_user);
            next.run(req).await
        }
        Err(e) => {
            tracing::debug!("AuthGuard: token invalid - {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": { "code": 401, "message": "Token 无效或已过期" }
                })),
            )
                .into_response()
        }
    }
}
