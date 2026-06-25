/// 应用配置
/// 约定：配置集中管理，支持多环境覆盖
pub struct AppConfig {
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    /// 加载配置（后续会支持从文件 + 环境变量加载）
    pub fn load() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}
