//! WebSocket 会话管理

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

/// WebSocket 消息（简化封装）
#[derive(Debug, Clone)]
pub enum WsMessage {
    /// 文本消息
    Text(String),
    /// 二进制消息
    Binary(Vec<u8>),
    /// Ping
    Ping,
    /// 关闭
    Close,
}

impl From<Message> for WsMessage {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Text(t) => WsMessage::Text(t.to_string()),
            Message::Binary(b) => WsMessage::Binary(b.to_vec()),
            Message::Ping(_) => WsMessage::Ping,
            Message::Close(_) => WsMessage::Close,
            _ => WsMessage::Close,
        }
    }
}

impl From<WsMessage> for Message {
    fn from(msg: WsMessage) -> Self {
        match msg {
            WsMessage::Text(t) => Message::Text(t.into()),
            WsMessage::Binary(b) => Message::Binary(b.into()),
            WsMessage::Ping => Message::Ping(vec![].into()),
            WsMessage::Close => Message::Close(None),
        }
    }
}

/// WebSocket 会话
/// 每个客户端连接对应一个 Session，提供发送消息的能力
pub struct WsSession {
    tx: mpsc::UnboundedSender<Message>,
    id: String,
}

impl WsSession {
    pub fn new(tx: mpsc::UnboundedSender<Message>, id: String) -> Self {
        Self { tx, id }
    }

    /// 获取连接 ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 发送消息给客户端
    pub async fn send(&self, msg: WsMessage) {
        let _ = self.tx.send(msg.into());
    }

    /// 发送文本消息
    pub async fn send_text(&self, text: impl Into<String>) {
        let _ = self.tx.send(Message::Text(text.into().into()));
    }

    /// 发送 JSON
    pub async fn send_json<T: serde::Serialize>(&self, data: &T) {
        if let Ok(json) = serde_json::to_string(data) {
            let _ = self.tx.send(Message::Text(json.into()));
        }
    }
}

/// 处理 WebSocket 连接的内部循环
pub(crate) async fn handle_socket(
    socket: WebSocket,
    handler: Arc<dyn super::WsHandler>,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let session_id = uuid_v4();
    let session = Arc::new(WsSession::new(tx, session_id.clone()));

    // 通知连接建立
    handler.on_connect(&session).await;

    // 发送任务：从 channel 接收消息发给客户端
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 接收任务：从客户端接收消息分发给 handler
    let handler_clone = handler.clone();
    let session_clone = session.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Close(_) => break,
                _ => {
                    let ws_msg = WsMessage::from(msg);
                    handler_clone.on_message(&session_clone, ws_msg).await;
                }
            }
        }
    });

    // 等待任一任务结束
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // 通知连接断开
    handler.on_disconnect(&session).await;
    tracing::debug!("WebSocket session {} disconnected", session_id);
}

/// 简单的 UUID v4 生成（不引入额外依赖）
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", ts)
}
