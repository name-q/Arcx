//! WebSocket 支持
//!
//! 框架约定：
//! - 实现 WsHandler trait 定义 WebSocket 处理器
//! - 通过 WsRegistry 注册并生成路由
//! - 框架自动处理连接升级和消息循环
//!
//! 用法：
//! ```rust
//! pub struct ChatWs;
//!
//! #[async_trait]
//! impl WsHandler for ChatWs {
//!     fn path(&self) -> &str { "/ws/chat" }
//!
//!     async fn on_message(&self, session: &WsSession, msg: WsMessage) {
//!         session.send_text("echo: ...").await;
//!     }
//! }
//! ```

pub mod handler;
pub mod session;

use std::sync::Arc;
use axum::Router;

use crate::context::AppState;
pub use handler::WsHandler;
pub use session::{WsMessage, WsSession};

/// WebSocket 路由注册器
pub struct WsRegistry {
    handlers: Vec<Arc<dyn WsHandler>>,
}

impl WsRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// 注册一个 WebSocket handler
    pub fn register(&mut self, ws_handler: impl WsHandler + 'static) {
        let path = ws_handler.path().to_string();
        tracing::info!("  WebSocket registered: {}", path);
        self.handlers.push(Arc::new(ws_handler));
    }

    /// 生成所有 WebSocket 路由
    pub fn into_router(self) -> Router<AppState> {
        let mut router = Router::new();
        for ws_handler in self.handlers {
            let path = ws_handler.path().to_string();
            let method_router = handler::ws_upgrade_handler(ws_handler);
            router = router.route(&path, method_router);
        }
        router
    }
}
