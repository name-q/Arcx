//! 示例：Redis PubSub 客户端
//!
//! 展示如何基于 Client trait 实现一个发布订阅客户端
//! 这不是真实的 Redis 实现，仅展示框架约定
//!
//! ```rust
//! // 注册客户端
//! client_registry.register(
//!     Arc::new(RedisPubSubClient::new("redis://127.0.0.1:6379")),
//!     Some(Duration::from_secs(10)),
//! ).await?;
//!
//! // 在 Service 中使用 Stream 消费消息
//! let client = ctx.resource::<RedisPubSubClient>().unwrap();
//! let mut stream = client.subscribe("order.created").await?;
//! while let Some(msg) = stream.next().await {
//!     println!("Got: {:?}", msg);
//! }
//! ```

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::Stream;

use super::{Client, ClientError, ClientErrorKind, Subscriber};

/// 示例：内存版 PubSub 客户端
/// 实际项目中替换为真实的 Redis/NATS/Kafka 实现
pub struct MemoryPubSubClient {
    name: String,
    connected: AtomicBool,
    /// 内部用 broadcast channel 实现发布订阅
    /// 真实实现中这里是 Redis connection
    tx: broadcast::Sender<PubSubMessage>,
}

#[derive(Debug, Clone)]
pub struct PubSubMessage {
    pub topic: String,
    pub payload: String,
}

impl MemoryPubSubClient {
    pub fn new(name: impl Into<String>, capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            name: name.into(),
            connected: AtomicBool::new(false),
            tx,
        }
    }
}

#[async_trait]
impl Client for MemoryPubSubClient {
    fn name(&self) -> &str {
        &self.name
    }

    async fn connect(&self) -> Result<(), ClientError> {
        // 真实实现：建立 TCP 连接
        self.connected.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), ClientError> {
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    async fn health_check(&self) -> Result<bool, ClientError> {
        // 真实实现：发送 PING 命令
        Ok(self.is_connected())
    }
}

#[async_trait]
impl Subscriber for MemoryPubSubClient {
    type Item = PubSubMessage;

    /// 订阅主题 —— 返回 Stream 而不是注册回调
    /// 这是关键设计：消费者拿到 Stream 后可以:
    /// - while let Some(msg) = stream.next().await 循环消费
    /// - select! 与其他 Future 组合
    /// - 用 StreamExt 的 filter/map/take 等组合子处理
    async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Item> + Send>>, ClientError> {
        if !self.is_connected() {
            return Err(ClientError::disconnected(&self.name, "not connected"));
        }

        let rx = self.tx.subscribe();
        let topic = topic.to_string();

        // 用 BroadcastStream 包装为 Stream trait
        // 然后 filter 只留下匹配 topic 的消息
        let stream = BroadcastStream::new(rx);
        let filtered = tokio_stream::StreamExt::filter_map(stream, move |result| {
            match result {
                Ok(msg) if msg.topic == topic => Some(msg),
                _ => None,
            }
        });

        Ok(Box::pin(filtered))
    }

    /// 发布消息（所有订阅者都会收到）
    async fn publish(&self, topic: &str, data: Self::Item) -> Result<(), ClientError> {
        if !self.is_connected() {
            return Err(ClientError::disconnected(&self.name, "not connected"));
        }

        let msg = PubSubMessage {
            topic: topic.to_string(),
            payload: data.payload,
        };
        let _ = self.tx.send(msg);
        Ok(())
    }
}
