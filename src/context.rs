use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::config::AppConfig;

/// 应用共享状态
/// 通过 Arc 在多个请求间共享，零拷贝
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(config: AppConfig, db: DatabaseConnection) -> Self {
        Self {
            config: Arc::new(config),
            db,
        }
    }
}

/// 请求上下文
/// 每个 handler 可以直接提取，自动获得 config、db 等访问能力
/// 这是贯穿 Controller / Service 的核心对象
pub struct Context {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
}

/// 让 Context 能从请求中自动提取
/// Controller 写法：pub async fn info(ctx: Context, ...) -> Result<...>
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
            db: app_state.db,
        })
    }
}
