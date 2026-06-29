//! 插件系统
//! 
//! 约定：
//! - 插件实现 Plugin trait
//! - 通过 PluginManager 统一管理生命周期
//! - 配置文件 [plugin.xxx] 段作为插件配置
//! - 插件可向 Context 注入资源
//! - 内置插件通过 enable = true 启用，无需手动注册

pub mod builtin;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// 插件 trait —— 所有插件必须实现
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// 插件名称（用于日志和配置索引）
    fn name(&self) -> &str;

    /// 初始化插件
    /// config 参数为该插件对应的配置段 [plugin.{name}]
    async fn init(&mut self, config: &toml::Value) -> Result<(), PluginError>;

    /// 关闭插件，释放资源
    async fn shutdown(&self) -> Result<(), PluginError> {
        Ok(())
    }

    /// 获取插件提供的资源
    /// 返回后会被注入到 AppState，Controller 通过 ctx.resource::<T>() 访问
    fn resource(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        None
    }
}

/// 插件错误
#[derive(Debug)]
pub struct PluginError {
    pub plugin: String,
    pub message: String,
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plugin [{}] error: {}", self.plugin, self.message)
    }
}

impl PluginError {
    pub fn new(plugin: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            plugin: plugin.into(),
            message: message.into(),
        }
    }
}

/// 插件管理器
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    resources: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            resources: HashMap::new(),
        }
    }

    /// 手动注册插件（用于自定义插件）
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        tracing::info!("  Plugin registered: {}", plugin.name());
        self.plugins.push(plugin);
    }

    /// 自动注册内置插件（根据配置中的 enable 字段）
    /// 扫描 [plugin.*] 段，对已知的内置插件自动创建实例
    pub fn auto_register_builtin(&mut self, config: &toml::Value) {
        let plugin_section = match config.get("plugin").and_then(|v| v.as_table()) {
            Some(t) => t,
            None => return,
        };

        for (name, plugin_config) in plugin_section {
            let enabled = plugin_config
                .get("enable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !enabled {
                continue;
            }

            match name.as_str() {
                "database" => {
                    self.register(Box::new(builtin::database::DatabasePlugin::new()));
                }
                "jwt" => {
                    self.register(Box::new(builtin::jwt::JwtPlugin::new()));
                }
                _ => {
                    tracing::warn!("  Unknown plugin in config: {}", name);
                }
            }
        }
    }

    /// 初始化所有已注册的插件
    pub async fn init_all(&mut self, config: &toml::Value) -> Result<(), PluginError> {
        if self.plugins.is_empty() {
            tracing::info!("No plugins registered");
            return Ok(());
        }

        for plugin in self.plugins.iter_mut() {
            let plugin_name = plugin.name().to_string();

            // 从配置中提取: [plugin.{name}]
            let plugin_config = config
                .get("plugin")
                .and_then(|p| p.get(&plugin_name))
                .cloned()
                .unwrap_or(toml::Value::Table(toml::map::Map::new()));

            tracing::info!("  Plugin initializing: {}", plugin_name);
            plugin.init(&plugin_config).await?;
            tracing::info!("  Plugin ready: {} ✓", plugin_name);

            // 收集插件资源
            if let Some(resource) = plugin.resource() {
                let type_id = (*resource).type_id();
                self.resources.insert(type_id, resource);
            }
        }

        tracing::info!("All plugins initialized ({} total)", self.plugins.len());
        Ok(())
    }

    /// 关闭所有插件（逆序）
    pub async fn shutdown_all(&self) {
        for plugin in self.plugins.iter().rev() {
            if let Err(e) = plugin.shutdown().await {
                tracing::error!("Plugin shutdown error: {}", e);
            }
        }
        tracing::info!("All plugins shut down");
    }

    /// 取走资源（转移所有权给 AppState）
    pub fn take_resources(&mut self) -> HashMap<TypeId, Arc<dyn Any + Send + Sync>> {
        std::mem::take(&mut self.resources)
    }
}
