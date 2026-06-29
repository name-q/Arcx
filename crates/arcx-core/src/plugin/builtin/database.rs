//! 数据库插件
//! 
//! 配置方式：
//! ```toml
//! [plugin.database]
//! enable = true
//! url = "sqlite:./data.db?mode=rwc"
//! max_connections = 10
//! min_connections = 1
//! ```
//! 
//! Controller 中通过 Context 获取连接：
//! ```rust
//! let db = ctx.resource::<DatabaseConnection>().unwrap();
//! ```

use std::sync::Arc;
use std::any::Any;
use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use crate::plugin::{Plugin, PluginError};

/// 数据库插件
pub struct DatabasePlugin {
    connection: Option<DatabaseConnection>,
}

impl DatabasePlugin {
    pub fn new() -> Self {
        Self { connection: None }
    }
}

#[async_trait::async_trait]
impl Plugin for DatabasePlugin {
    fn name(&self) -> &str {
        "database"
    }

    async fn init(&mut self, config: &toml::Value) -> Result<(), PluginError> {
        let url = config
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("sqlite:./data.db?mode=rwc");

        let max_conn = config
            .get("max_connections")
            .and_then(|v| v.as_integer())
            .unwrap_or(10) as u32;

        let min_conn = config
            .get("min_connections")
            .and_then(|v| v.as_integer())
            .unwrap_or(1) as u32;

        let mut opt = ConnectOptions::new(url.to_string());
        opt.max_connections(max_conn)
            .min_connections(min_conn)
            .sqlx_logging(false);

        tracing::info!("    Database connecting: {}", url);

        let conn = Database::connect(opt)
            .await
            .map_err(|e| PluginError::new("database", format!("Connection failed: {}", e)))?;

        tracing::info!("    Database connected ✓");
        self.connection = Some(conn);
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), PluginError> {
        if let Some(conn) = &self.connection {
            conn.clone().close().await.map_err(|e| {
                PluginError::new("database", format!("Close failed: {}", e))
            })?;
            tracing::info!("    Database connection closed");
        }
        Ok(())
    }

    fn resource(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.connection
            .as_ref()
            .map(|conn| Arc::new(conn.clone()) as Arc<dyn Any + Send + Sync>)
    }
}
