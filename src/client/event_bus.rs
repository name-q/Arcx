//! 事件总线 —— 基于 tokio broadcast channel
//!
//! 用途：
//! - 框架级事件广播（启动完成、配置变更、插件加载等）
//! - 业务事件发布/订阅
//! - 替代 Arc<Vec<Callback>> 的事件监听模式
//!
//! 设计选择：
//! - broadcast channel：多个消费者都能收到同一条消息
//! - 消费者收到的是 Stream，可以 select! 组合其他异步操作
//! - 事件带类型标签，消费者按需过滤
//!
//! 用法：
//! ```rust
//! let bus = EventBus::new(128);
//!
//! // 发送事件（非阻塞，不需要 await）
//! bus.emit(AppEvent::Started);
//!
//! // 订阅事件（返回 Stream）
//! let mut stream = bus.subscribe();
//! tokio::spawn(async move {
//!     while let Some(event) = stream.recv().await {
//!         match event {
//!             AppEvent::Started => println!("App started!"),
//!             _ => {}
//!         }
//!     }
//! });
//! ```

use tokio::sync::broadcast;

/// 框架级事件类型
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// 应用启动完成
    Started,
    /// 应用即将关闭
    Stopping,
    /// 配置重新加载
    ConfigReloaded,
    /// 插件加载完成
    PluginLoaded { name: String },
    /// 插件关闭
    PluginStopped { name: String },
    /// 客户端连接建立
    ClientConnected { name: String },
    /// 客户端连接断开
    ClientDisconnected { name: String },
    /// 健康检查失败
    HealthCheckFailed { name: String, reason: String },
    /// 自定义事件（扩展用）
    Custom { kind: String, payload: String },
}

/// 事件总线
/// 基于 tokio::sync::broadcast，支持多生产者多消费者
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<AppEvent>,
}

impl EventBus {
    /// 创建事件总线
    /// capacity: 缓冲区大小（消费者来不及消费时的积压上限）
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// 发送事件（同步，不阻塞）
    /// 如果没有订阅者，消息会被丢弃（不报错）
    pub fn emit(&self, event: AppEvent) {
        // send 返回 Err 只说明没有 receiver，不影响逻辑
        let _ = self.tx.send(event);
    }

    /// 订阅事件流
    /// 返回一个 Receiver，可以循环 recv() 消费
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.tx.subscribe()
    }

    /// 获取当前订阅者数量
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(128)
    }
}
