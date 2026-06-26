//! Echo WebSocket 示例
//! 路径: /ws/echo
//! 演示 WebSocket 框架用法

use crate::ws::{WsHandler, WsMessage, WsSession};

pub struct EchoWs;

#[async_trait::async_trait]
impl WsHandler for EchoWs {
    fn path(&self) -> &str {
        "/ws/echo"
    }

    async fn on_connect(&self, session: &WsSession) {
        tracing::info!("[EchoWs] client connected: {}", session.id());
        session.send_text("Welcome to Arcx WebSocket!").await;
    }

    async fn on_message(&self, session: &WsSession, msg: WsMessage) {
        match msg {
            WsMessage::Text(text) => {
                tracing::info!("[EchoWs] received: {}", text);
                session.send_text(format!("echo: {}", text)).await;
            }
            WsMessage::Binary(data) => {
                session.send(WsMessage::Binary(data)).await;
            }
            _ => {}
        }
    }

    async fn on_disconnect(&self, session: &WsSession) {
        tracing::info!("[EchoWs] client disconnected: {}", session.id());
    }
}
