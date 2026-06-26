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

/// 顶层配置结构
/// 框架约定：config/ 目录下按环境放置 TOML 配置文件
/// - config.default.toml  基础配置
/// - config.dev.toml      开发环境覆盖
/// - config.prod.toml     生产环境覆盖
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub app: AppInfo,
}

impl AppConfig {
    /// 加载配置
    /// 策略：default → env 覆盖合并
    /// 环境由 ARCX_ENV 环境变量决定（默认 dev）
    pub fn load() -> Self {
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

        toml::from_str(&config_str).expect("Failed to parse config")
    }
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
