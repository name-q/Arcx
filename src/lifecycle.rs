//! 应用生命周期管理
//! 提供启动/关闭钩子，优雅退出支持

use crate::plugin::PluginManager;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 生命周期钩子类型
type HookFn = Box<dyn Fn() + Send + Sync>;
type AsyncHookFn = Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;

/// 应用生命周期管理器
pub struct Lifecycle {
    on_start: Vec<HookFn>,
    on_shutdown: Vec<AsyncHookFn>,
    plugin_manager: Arc<RwLock<PluginManager>>,
}

impl Lifecycle {
    pub fn new(plugin_manager: Arc<RwLock<PluginManager>>) -> Self {
        Self {
            on_start: Vec::new(),
            on_shutdown: Vec::new(),
            plugin_manager,
        }
    }

    /// 注册启动钩子
    pub fn on_start(&mut self, hook: impl Fn() + Send + Sync + 'static) {
        self.on_start.push(Box::new(hook));
    }

    /// 注册关闭钩子
    pub fn on_shutdown(&mut self, hook: impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static) {
        self.on_shutdown.push(Box::new(hook));
    }

    /// 执行启动流程
    pub fn start(&self) {
        for hook in &self.on_start {
            hook();
        }
    }

    /// 执行关闭流程（包括插件关闭）
    pub async fn shutdown(&self) {
        tracing::info!("Graceful shutdown starting...");

        // 先执行用户注册的关闭钩子
        for hook in &self.on_shutdown {
            hook().await;
        }

        // 再关闭所有插件
        let pm = self.plugin_manager.read().await;
        pm.shutdown_all().await;

        tracing::info!("Graceful shutdown complete");
    }
}
