//! Session 存储后端抽象
//!
//! 默认使用 Cookie 存储（无需后端服务）。
//! 可扩展为 Redis / 内存 / 数据库存储。
//!
//! 当 session 数据量大或需要服务端管控时，
//! 切换到 Redis 存储：
//! ```toml
//! [session]
//! store = "redis"         # cookie(默认) / redis / memory
//! redis_url = "redis://127.0.0.1:6379"
//! ```

use async_trait::async_trait;
use std::collections::HashMap;

/// Session 存储后端 trait
/// 实现此 trait 即可扩展新的存储方式
#[async_trait]
pub trait SessionStore: Send + Sync + 'static {
    /// 读取 session 数据
    async fn load(&self, session_id: &str) -> Option<HashMap<String, serde_json::Value>>;

    /// 保存 session 数据
    async fn save(
        &self,
        session_id: &str,
        data: &HashMap<String, serde_json::Value>,
        max_age_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 删除 session
    async fn destroy(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 内存存储（开发/测试用）
/// 注意：进程重启后丢失，不适合生产
pub struct MemoryStore {
    data: tokio::sync::RwLock<HashMap<String, HashMap<String, serde_json::Value>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            data: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SessionStore for MemoryStore {
    async fn load(&self, session_id: &str) -> Option<HashMap<String, serde_json::Value>> {
        let store = self.data.read().await;
        store.get(session_id).cloned()
    }

    async fn save(
        &self,
        session_id: &str,
        data: &HashMap<String, serde_json::Value>,
        _max_age_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut store = self.data.write().await;
        store.insert(session_id.to_string(), data.clone());
        Ok(())
    }

    async fn destroy(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut store = self.data.write().await;
        store.remove(session_id);
        Ok(())
    }
}
