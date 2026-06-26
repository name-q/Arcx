use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use std::sync::Arc;

use crate::config::AppConfig;

/// 应用共享状态
/// 通过 Clone（内部 Arc）在多个请求间共享
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    // 插件注入的资源会加在这里（db、redis 等）
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

/// 请求上下文 Context
/// Controller handler 的第一个参数，自动从请求中提取
/// 提供对 config、插件资源的便捷访问
///
/// 用法：
/// ```rust
/// pub async fn index(ctx: Context) -> impl IntoResponse { ... }
/// ```
pub struct Context {
    pub config: Arc<AppConfig>,
}

impl Context {
    /// 获取当前环境名
    pub fn env(&self) -> &str {
        &self.config.app.env
    }

    /// 是否为开发环境
    pub fn is_dev(&self) -> bool {
        self.config.app.env == "dev"
    }
}

/// 实现 FromRequestParts，让 Context 能作为 handler 参数自动提取
#[axum::async_trait]
impl<S> FromRequestParts<S> for Context
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        Ok(Context {
            config: app_state.config,
        })
    }
}
