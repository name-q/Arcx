use serde::Deserialize;

use std::fs;
use std::path::Path;

/// 服务器配置
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// 应用信息
#[derive(Debug, Deserialize, Clone)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default)]
    pub debug: bool,
}

fn default_env() -> String {
    "dev".to_string()
}

/// 中间件配置
#[derive(Debug, Deserialize, Clone)]
pub struct MiddlewareConfig {
    #[serde(default = "default_true")]
    pub cors: bool,
    #[serde(default = "default_true")]
    pub logger: bool,
}

fn default_true() -> bool {
    true
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            cors: true,
            logger: true,
        }
    }
}

/// 顶层配置结构（强类型）
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub app: AppInfo,
    #[serde(default)]
    pub middleware: MiddlewareConfig,
}

impl AppConfig {
    /// 加载配置（强类型解析）
    pub fn load() -> Self {
        let raw = load_raw_str();
        toml::from_str(&raw).expect("Failed to parse config")
    }

    /// 判断某个中间件是否启用
    pub fn middleware_enabled(&self, name: &str) -> bool {
        match name {
            "cors" => self.middleware.cors,
            "logger" => self.middleware.logger,
            _ => false,
        }
    }
}

/// 加载原始配置为 toml::Value
/// 用于传递给插件系统（插件只需要自己的配置段）
pub fn load_raw_config() -> toml::Value {
    let raw = load_raw_str();
    toml::from_str(&raw).expect("Failed to parse raw config")
}

/// 加载并合并配置文件为字符串
fn load_raw_str() -> String {
    let env = std::env::var("ARCX_ENV").unwrap_or_else(|_| "dev".to_string());
    tracing::info!("Loading config for environment: {}", env);

    // 读取默认配置
    let default_path = "config/config.default.toml";
    let mut config_str = fs::read_to_string(default_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", default_path));

    // 读取环境配置并合并
    let env_path = format!("config/config.{}.toml", env);
    if Path::new(&env_path).exists() {
        let env_str = fs::read_to_string(&env_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", env_path));
        config_str = merge_toml(&config_str, &env_str);
    }

    config_str
}

/// TOML 合并：环境配置覆盖默认配置
fn merge_toml(base: &str, overlay: &str) -> String {
    let mut base_value: toml::Value = toml::from_str(base).expect("Invalid base TOML");
    let overlay_value: toml::Value = toml::from_str(overlay).expect("Invalid overlay TOML");

    merge_value(&mut base_value, &overlay_value);
    toml::to_string(&base_value).expect("Failed to serialize merged config")
}

/// 递归合并 TOML Value
fn merge_value(base: &mut toml::Value, overlay: &toml::Value) {
    match (base, overlay) {
        (toml::Value::Table(base_table), toml::Value::Table(overlay_table)) => {
            for (key, value) in overlay_table {
                if let Some(base_val) = base_table.get_mut(key) {
                    merge_value(base_val, value);
                } else {
                    base_table.insert(key.clone(), value.clone());
                }
            }
        }
        (base, overlay) => {
            *base = overlay.clone();
        }
    }
}
