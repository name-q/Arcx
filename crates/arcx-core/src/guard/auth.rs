//! 鉴权守卫 — 基于 AuthProvider trait 的通用实现
//!
//! 框架只定义接口，用户实现具体验证逻辑。
//! 中间件行为：
//! - 放行：调用 provider.authenticate() 成功 → 注入 AuthUser → next
//! - 阻断：authenticate 返回 Err → 直接返回错误响应

use axum::{
    extract::{Request, State},
    http,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::context::AppState;
use crate::error::AppError;

/// 鉴权用户 — authenticate 成功后注入请求
///
/// handler 参数中写 `user: AuthUser` 即可提取。
/// 写 `user: Option<AuthUser>` 则不报错，未登录时为 None。
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// 用户唯一标识
    pub id: String,
    /// 用户附加数据（角色、权限等，由 AuthProvider 决定内容）
    pub payload: serde_json::Value,
}

/// 鉴权提供者 trait — 用户必须实现
///
/// 框架不关心你用 JWT/Session/OAuth/远程验证：
/// - token 从哪取（header/cookie/query）由你决定
/// - 怎么验证由你决定
/// - 返回 AuthUser 即表示通过
///
/// ## 示例
///
/// ```rust
/// pub struct JwtAuth {
///     secret: String,
/// }
///
/// #[async_trait]
/// impl AuthProvider for JwtAuth {
///     async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError> {
///         let token = parts.headers
///             .get("Authorization")
///             .and_then(|v| v.to_str().ok())
///             .and_then(|v| v.strip_prefix("Bearer "))
///             .ok_or(AppError::unauthorized("Missing token"))?;
///
///         let claims = verify_jwt(token, &self.secret)
///             .map_err(|_| AppError::unauthorized("Invalid token"))?;
///
///         Ok(AuthUser {
///             id: claims.sub,
///             payload: json!({ "role": claims.role }),
///         })
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait AuthProvider: Send + Sync + 'static {
    /// 从请求中提取并验证身份
    ///
    /// - 成功：返回 AuthUser
    /// - 失败：返回 AppError（通常是 unauthorized/forbidden）
    async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError>;
}

/// 请求元数据（传给 AuthProvider 用于提取 token）
///
/// 包含 headers、URI、method 等，不含 body。
pub struct RequestParts {
    pub headers: http::HeaderMap,
    pub uri: http::Uri,
    pub method: http::Method,
}

/// 实现 FromRequestParts，让 AuthUser 能作为 handler 参数直接提取
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthUser
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
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| {
                AppError::unauthorized("Not authenticated").into_response()
            })
    }
}

/// 鉴权守卫中间件（框架内部使用）
///
/// guarded_scope 自动挂载此中间件。
/// 行为：
/// 1. 从 AppState 获取用户注册的 AuthProvider
/// 2. 调用 provider.authenticate(parts)
/// 3. 成功 → AuthUser 注入请求扩展 → 放行
/// 4. 失败 → 阻断，返回错误响应
pub async fn auth_guard(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    // 获取用户注册的 AuthProvider
    let provider = match state.auth_provider() {
        Some(p) => p,
        None => {
            tracing::error!("guarded_scope used but no AuthProvider registered. Call Arcx::new().auth(provider) first.");
            return AppError::internal("Auth not configured").into_response();
        }
    };

    // 构造 RequestParts
    let parts = RequestParts {
        headers: req.headers().clone(),
        uri: req.uri().clone(),
        method: req.method().clone(),
    };

    // 调用用户实现的验证逻辑
    match provider.authenticate(&parts).await {
        Ok(user) => {
            // 放行：注入 AuthUser，继续执行 controller
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Err(e) => {
            // 阻断：直接返回错误响应
            e.into_response()
        }
    }
}
