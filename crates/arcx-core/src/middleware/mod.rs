//! 中间件层
//!
//! 每个中间件独立配置，配置了就生效：
//!
//! ```toml
//! [cors]
//! enable = true
//! allowed_origins = ["*"]
//!
//! [request_logger]
//! enable = true
//!
//! [security]
//! enable = true
//! csrf = false
//! ```

pub mod cors;
pub mod request_logger;
pub mod security;

use axum::{middleware, Router};
use crate::config::AppConfig;
use crate::context::AppState;
use crate::logger::trace_id::trace_id_middleware;

/// 注册全局中间件
/// 根据各自配置段决定开启哪些中间件
/// 注册顺序（从外到内）：security → cors → logger → trace_id
pub fn apply_global_middleware(router: Router<AppState>, config: &AppConfig) -> Router<AppState> {
    let mut app = router;

    // Trace ID（始终启用，最内层）
    app = app.layer(middleware::from_fn(trace_id_middleware));
    tracing::info!("  Middleware enabled: trace_id");

    // 请求日志
    if config.request_logger.enable {
        app = app.layer(middleware::from_fn(request_logger::request_logger));
        tracing::info!("  Middleware enabled: request_logger");
    }

    // CORS
    if config.cors.enable {
        app = app.layer(cors::cors_layer_from_config(&config.cors));
        tracing::info!("  Middleware enabled: cors");
    }

    // 安全中间件
    let security_config = config.security.clone().unwrap_or_default();
    if security_config.enable {
        security::apply_security_middleware_with_config(&mut app, &security_config);
    }

    app
}
