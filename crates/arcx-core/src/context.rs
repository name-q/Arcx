//! 应用共享状态（App 级）
//!
//! AppState 是整个应用生命周期的共享数据容器。
//! 不同于 Ctx（请求级），AppState 跨所有请求存在。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::client::event_bus::EventBus;
use crate::config::watcher::ConfigWatcher;
use crate::config::AppConfig;
use crate::guard::AuthProvider;

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
    /// 用户注册的鉴权提供者（可选）
    auth_provider: Option<Arc<dyn AuthProvider>>,
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
            auth_provider: None,
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
            auth_provider: None,
        }
    }

    /// 设置鉴权提供者
    pub fn set_auth_provider(&mut self, provider: impl AuthProvider) {
        self.auth_provider = Some(Arc::new(provider));
    }

    /// 设置鉴权提供者（Box 版本，框架内部使用）
    pub fn set_auth_provider_boxed(&mut self, provider: Box<dyn AuthProvider>) {
        self.auth_provider = Some(Arc::from(provider));
    }

    /// 获取鉴权提供者（guard 中间件内部调用）
    pub fn auth_provider(&self) -> Option<Arc<dyn AuthProvider>> {
        self.auth_provider.clone()
    }

    /// 获取插件资源
    /// 用法: state.resource::<DatabaseConnection>()
    pub fn resource<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.clone().downcast::<T>().ok())
    }

    /// 获取资源池引用（Ctx 内部使用）
    pub(crate) fn resources_ref(&self) -> &Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>> {
        &self.resources
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
