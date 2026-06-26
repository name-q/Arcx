//! WebSocket Handler trait 与连接升级处理

use std::sync::Arc;
use axum::{
    extract::ws::WebSocketUpgrade,
    response::IntoResponse,
};

use super::session::{self, WsMessage, WsSession};

/// WebSocket Handler trait
/// 实现此 trait 来处理 WebSocket 连接
#[async_trait::async_trait]
pub trait WsHandler: Send + Sync {
    /// WebSocket 路径（如 "/ws/chat"）
    fn path(&self) -> &str;

    /// 连接建立时触发
    async fn on_connect(&self, session: &WsSession) {
        tracing::debug!("WS connected: {}", session.id());
    }

    /// 收到消息时触发
    async fn on_message(&self, session: &WsSession, msg: WsMessage);

    /// 连接断开时触发
    async fn on_disconnect(&self, session: &WsSession) {
        tracing::debug!("WS disconnected: {}", session.id());
    }
}

/// WebSocket 升级 handler 工厂
/// 返回 axum 兼容的 handler
pub fn ws_upgrade_handler(
    handler: Arc<dyn WsHandler>,
) -> axum::routing::MethodRouter<crate::context::AppState> {
    axum::routing::get(move |ws: WebSocketUpgrade| {
        let handler = handler.clone();
        async move {
            ws.on_upgrade(move |socket| session::handle_socket(socket, handler))
        }
    })
}
