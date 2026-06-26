use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::AppConfig;

/// 应用共享状态
/// 贯穿整个应用生命周期，多线程共享
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    /// 插件注入的资源池（类型安全的 Any 容器）
    resources: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
            resources: Arc::new(HashMap::new()),
        }
    }

    /// 带资源创建（由 PluginManager 初始化后注入）
    pub fn with_resources(
        config: AppConfig,
        resources: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            resources: Arc::new(resources),
        }
    }

    /// 获取插件资源
    /// 用法: state.resource::<DatabaseConnection>()
    pub fn resource<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.clone().downcast::<T>().ok())
    }
}

/// 请求上下文 Context
/// Controller handler 的第一个参数，自动从请求中提取
///
/// 用法：
/// ```rust
/// pub async fn index(ctx: Context) -> impl IntoResponse {
///     let db = ctx.resource::<DbPool>().unwrap();
/// }
/// ```
pub struct Context {
    pub config: Arc<AppConfig>,
    resources: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
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

    /// 获取插件注入的资源
    /// 类型安全：编译期确定类型
    pub fn resource<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.clone().downcast::<T>().ok())
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
            resources: app_state.resources,
        })
    }
}
