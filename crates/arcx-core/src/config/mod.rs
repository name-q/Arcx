//! 配置管理
//!
//! - 多环境配置加载（TOML，default → env 合并覆盖）
//! - 配置热更新通道（watch channel，详见 watcher 模块）
//! - 动态配置访问（dot-notation 路径索引，如 "redis.url"）

pub mod watcher;

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
    #[serde(default = "default_true")]
    pub security: bool,
}

fn default_true() -> bool {
    true
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            cors: true,
            logger: true,
            security: true,
        }
    }
}

/// 顶层配置结构（强类型 + 动态扩展）
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub app: AppInfo,
    #[serde(default)]
    pub middleware: MiddlewareConfig,
    #[serde(default)]
    pub logger: crate::logger::LoggerConfig,
    #[serde(default)]
    pub httpclient: crate::httpclient::HttpClientConfig,
    #[serde(default)]
    pub session: Option<crate::session::SessionConfig>,
    #[serde(default)]
    pub security: Option<crate::middleware::security::SecurityConfig>,

    /// 原始配置（保留所有字段，支持动态索引）
    #[serde(skip)]
    raw: Option<toml::Value>,
}

impl AppConfig {
    /// 加载配置（强类型解析 + 保留原始值）
    pub fn load() -> Self {
        let raw_str = load_raw_str();
        let raw_value: toml::Value = toml::from_str(&raw_str).expect("Failed to parse config");
        let mut config: AppConfig = toml::from_str(&raw_str).expect("Failed to parse config");
        config.raw = Some(raw_value);
        config
    }

    /// 判断某个中间件是否启用
    pub fn middleware_enabled(&self, name: &str) -> bool {
        match name {
            "cors" => self.middleware.cors,
            "logger" => self.middleware.logger,
            "security" => self.middleware.security,
            _ => false,
        }
    }

    /// 通过 dot-notation 路径获取配置值
    ///
    /// 安全的动态配置访问，支持任意嵌套层级：
    /// ```rust
    /// // config.toml:
    /// // [redis]
    /// // url = "redis://localhost:6379"
    /// // pool_size = 10
    ///
    /// let url: Option<String> = config.get("redis.url");
    /// let pool: Option<i64> = config.get("redis.pool_size");
    /// let port: Option<i64> = config.get("server.port");
    /// ```
    ///
    /// 支持的类型：String, i64, f64, bool
    pub fn get<T: FromTomlValue>(&self, path: &str) -> Option<T> {
        let raw = self.raw.as_ref()?;
        let value = resolve_path(raw, path)?;
        T::from_toml_value(value)
    }

    /// 获取配置中某个段（table）作为 toml::Value
    ///
    /// 适合插件读取自己整段配置：
    /// ```rust
    /// let redis_config = config.get_section("redis");
    /// ```
    pub fn get_section(&self, key: &str) -> Option<&toml::Value> {
        let raw = self.raw.as_ref()?;
        resolve_path(raw, key)
    }

    /// 将某个配置段反序列化为自定义结构
    ///
    /// ```rust
    /// #[derive(Deserialize)]
    /// struct RedisConfig { url: String, pool_size: u32 }
    ///
    /// let redis: Option<RedisConfig> = config.get_as("redis");
    /// ```
    pub fn get_as<T: serde::de::DeserializeOwned>(&self, path: &str) -> Option<T> {
        let section = self.get_section(path)?;
        section.clone().try_into().ok()
    }
}

/// 按 dot-notation 路径解析 toml::Value
fn resolve_path<'a>(value: &'a toml::Value, path: &str) -> Option<&'a toml::Value> {
    let mut current = value;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    Some(current)
}

/// 从 toml::Value 提取具体类型的 trait
pub trait FromTomlValue: Sized {
    fn from_toml_value(value: &toml::Value) -> Option<Self>;
}

impl FromTomlValue for String {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_str().map(|s| s.to_string())
    }
}

impl FromTomlValue for i64 {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_integer()
    }
}

impl FromTomlValue for f64 {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_float()
    }
}

impl FromTomlValue for bool {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_bool()
    }
}

impl FromTomlValue for u16 {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_integer().and_then(|v| u16::try_from(v).ok())
    }
}

impl FromTomlValue for u32 {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_integer().and_then(|v| u32::try_from(v).ok())
    }
}

impl FromTomlValue for u64 {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_integer().and_then(|v| u64::try_from(v).ok())
    }
}

impl FromTomlValue for usize {
    fn from_toml_value(value: &toml::Value) -> Option<Self> {
        value.as_integer().and_then(|v| usize::try_from(v).ok())
    }
}

/// 加载原始配置为 toml::Value
/// 用于传递给插件系统（插件只需要自己的配置段）
pub fn load_raw_config() -> toml::Value {
    let raw = load_raw_str();
    toml::from_str(&raw).expect("Failed to parse raw config")
}

/// 加载并合并配置文件为字符串
///
/// 加载顺序（从低到高优先级）：
/// 1. config.default.toml 的 import 文件（按数组顺序）
/// 2. config.default.toml 本体字段
/// 3. config.{env}.toml 的 import 文件（按数组顺序）
/// 4. config.{env}.toml 本体字段
fn load_raw_str() -> String {
    let env = std::env::var("ARCX_ENV").unwrap_or_else(|_| "dev".to_string());

    // 读取默认配置
    let default_path = "config/config.default.toml";
    let default_str = fs::read_to_string(default_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", default_path));

    // 处理 default 的 import
    let config_str = process_imports(&default_str, default_path);

    // 读取环境配置并合并
    let env_path = format!("config/config.{}.toml", env);
    let config_str = if Path::new(&env_path).exists() {
        let env_str = fs::read_to_string(&env_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", env_path));
        // 处理 env 的 import
        let env_str = process_imports(&env_str, &env_path);
        merge_toml(&config_str, &env_str)
    } else {
        config_str
    };

    config_str
}

/// 处理配置文件中的 import 字段
///
/// import 声明的文件按顺序加载合并，最终与本体字段合并（本体优先级更高）。
/// import 的文件不存在时直接 panic 报错，防止粗心遗漏。
fn process_imports(toml_str: &str, source_file: &str) -> String {
    let value: toml::Value = toml::from_str(toml_str)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", source_file, e));

    // 提取 import 数组
    let imports = match value.get("import") {
        Some(toml::Value::Array(arr)) => arr
            .iter()
            .map(|v| {
                v.as_str()
                    .unwrap_or_else(|| panic!("import items must be strings in {}", source_file))
                    .to_string()
            })
            .collect::<Vec<_>>(),
        Some(_) => panic!("import must be an array of strings in {}", source_file),
        None => return toml_str.to_string(),
    };

    // 按顺序加载 import 文件并合并
    let mut merged = toml::Value::Table(toml::map::Map::new());
    for import_path in &imports {
        if !Path::new(import_path).exists() {
            panic!(
                "\n\x1b[31m✗ Config import error:\x1b[0m file \"{}\" not found (declared in {})\n",
                import_path, source_file
            );
        }
        let import_str = fs::read_to_string(import_path)
            .unwrap_or_else(|_| panic!("Failed to read imported config: {}", import_path));
        let import_value: toml::Value = toml::from_str(&import_str)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", import_path, e));
        merge_value(&mut merged, &import_value);
    }

    // 本体字段（去掉 import key）覆盖 import 内容
    let mut body = value.clone();
    if let toml::Value::Table(ref mut table) = body {
        table.remove("import");
    }
    merge_value(&mut merged, &body);

    toml::to_string(&merged).expect("Failed to serialize config after import processing")
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
