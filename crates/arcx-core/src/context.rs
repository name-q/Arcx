//! 应用状态与请求上下文

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::client::event_bus::EventBus;
use crate::config::watcher::ConfigWatcher;
use crate::config::AppConfig;

/// 应用共享状态
/// 贯穿整个应用生命周期，多线程共享
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    /// 插件注入的资源池（类型安全的 Any 容器）
    resources: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    /// 事件总线（broadcast channel 驱动）
    event_bus: EventBus,
    /// 配置热更新观察者（watch channel 驱动）
    config_watcher: ConfigWatcher,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let event_bus = EventBus::new(128);
        let (_, config_watcher) = ConfigWatcher::new(config.clone());
        Self {
            config: Arc::new(config),
            resources: Arc::new(HashMap::new()),
            event_bus,
            config_watcher,
        }
    }

    /// 带资源创建（由 PluginManager 初始化后注入）
    pub fn with_resources(
        config: AppConfig,
        resources: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
        event_bus: EventBus,
        config_watcher: ConfigWatcher,
    ) -> Self {
        Self {
            config: Arc::new(config),
            resources: Arc::new(resources),
            event_bus,
            config_watcher,
        }
    }

    /// 获取插件资源
    /// 用法: state.resource::<DatabaseConnection>()
    pub fn resource<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.clone().downcast::<T>().ok())
    }

    /// 获取事件总线
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// 获取配置观察者
    pub fn config_watcher(&self) -> &ConfigWatcher {
        &self.config_watcher
    }
}

/// 请求上下文 Context
/// Controller handler 的第一个参数，自动从请求中提取
///
/// 用法：
/// ```rust
/// pub async fn index(ctx: Context) -> impl IntoResponse {
///     let db = ctx.resource::<DbPool>().unwrap();
///     // 发送事件
///     ctx.emit(AppEvent::Custom { kind: "user.created".into(), payload: "{}".into() });
///     // 订阅配置变更
///     let config_rx = ctx.watch_config();
/// }
/// ```
pub struct Context {
    pub config: Arc<AppConfig>,
    resources: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    event_bus: EventBus,
    config_watcher: ConfigWatcher,
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

    /// 发送框架事件（非阻塞）
    pub fn emit(&self, event: crate::client::event_bus::AppEvent) {
        self.event_bus.emit(event);
    }

    /// 获取事件总线的订阅者
    pub fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<crate::client::event_bus::AppEvent> {
        self.event_bus.subscribe()
    }

    /// 获取配置变更 watcher
    /// 返回一个 Receiver，可以 .changed().await 等待配置变更
    pub fn watch_config(&self) -> tokio::sync::watch::Receiver<AppConfig> {
        self.config_watcher.subscribe()
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
            event_bus: app_state.event_bus,
            config_watcher: app_state.config_watcher,
        })
    }
}
