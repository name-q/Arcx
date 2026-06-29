//! Client 插件规范
//!
//! 约定：
//! - 第三方 SDK/长连接客户端实现 Client trait
//! - 框架管理客户端生命周期（connect → health_check → disconnect）
//! - 发布订阅类客户端实现 Subscriber trait，返回 Stream 而非回调
//! - 调用类客户端实现 Invoker trait，支持 async 调用
//!
//! Rust 不需要 Egg.js 那种多进程复用模式：
//! - 共享连接池用 Arc<Pool> 是正确的（池本身是共享资源）
//! - 数据推送用 tokio broadcast/watch channel（流式消费）
//! - 不用到处 Arc 包回调函数

pub mod event_bus;
pub mod example;
pub mod registry;

use async_trait::async_trait;
use std::pin::Pin;
use tokio_stream::Stream;

/// 客户端生命周期 trait
/// 所有外部连接类客户端必须实现
#[async_trait]
pub trait Client: Send + Sync + 'static {
    /// 客户端名称（用于日志/注册）
    fn name(&self) -> &str;

    /// 建立连接
    async fn connect(&self) -> Result<(), ClientError>;

    /// 健康检查（心跳、ping 等）
    async fn health_check(&self) -> Result<bool, ClientError> {
        Ok(true)
    }

    /// 优雅断开
    async fn disconnect(&self) -> Result<(), ClientError>;

    /// 是否已连接
    fn is_connected(&self) -> bool;
}

/// 发布订阅型客户端
/// 订阅返回 Stream，不是注册回调 —— 数据流动而非共享
#[async_trait]
pub trait Subscriber: Client {
    /// 消息类型
    type Item: Send + 'static;

    /// 订阅一个主题，返回消息流
    /// 调用方用 `while let Some(msg) = stream.next().await` 消费
    async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Item> + Send>>, ClientError>;

    /// 发布消息到主题
    async fn publish(&self, topic: &str, data: Self::Item) -> Result<(), ClientError>;
}

/// 调用型客户端（RPC 风格）
/// 请求-响应模式，适用于 gRPC、HTTP 远程调用等
#[async_trait]
pub trait Invoker: Client {
    /// 请求类型
    type Request: Send + 'static;
    /// 响应类型
    type Response: Send + 'static;

    /// 发起调用，等待响应
    async fn invoke(&self, req: Self::Request) -> Result<Self::Response, ClientError>;
}

/// 客户端错误
#[derive(Debug, Clone)]
pub struct ClientError {
    pub client: String,
    pub kind: ClientErrorKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ClientErrorKind {
    /// 连接失败
    ConnectionFailed,
    /// 连接超时
    Timeout,
    /// 连接断开
    Disconnected,
    /// 健康检查失败
    Unhealthy,
    /// 协议错误
    Protocol,
    /// 其他
    Other,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client [{}] {:?}: {}", self.client, self.kind, self.message)
    }
}

impl ClientError {
    pub fn new(
        client: impl Into<String>,
        kind: ClientErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            client: client.into(),
            kind,
            message: message.into(),
        }
    }

    pub fn connection_failed(client: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::new(client, ClientErrorKind::ConnectionFailed, msg)
    }

    pub fn timeout(client: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::new(client, ClientErrorKind::Timeout, msg)
    }

    pub fn disconnected(client: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::new(client, ClientErrorKind::Disconnected, msg)
    }
}
