//! 中间件层
//!
//! 约定：
//! - 全局中间件放在 src/middleware/ 目录
//! - 中间件通过配置文件控制开关
//! - 中间件执行顺序：外层先注册后执行（洋葱模型）
//!
//! 配置方式：
//! ```toml
//! [middleware]
//! cors = true
//! logger = true
//! security = true
//! ```

pub mod cors;
pub mod request_logger;
pub mod security;

use axum::{middleware, Router};
use crate::config::AppConfig;
use crate::context::AppState;
use crate::logger::trace_id::trace_id_middleware;

/// 注册全局中间件
/// 根据配置决定开启哪些中间件
/// 注册顺序（从外到内）：security → cors → logger → trace_id
pub fn apply_global_middleware(router: Router<AppState>, config: &AppConfig) -> Router<AppState> {
    let mut app = router;

    // Trace ID（始终启用，最内层）
    app = app.layer(middleware::from_fn(trace_id_middleware));
    tracing::info!("  Middleware enabled: trace_id");

    // 请求日志（默认启用）
    if config.middleware_enabled("logger") {
        app = app.layer(middleware::from_fn(request_logger::request_logger));
        tracing::info!("  Middleware enabled: logger");
    }

    // CORS（默认启用）
    if config.middleware_enabled("cors") {
        app = app.layer(cors::cors_layer());
        tracing::info!("  Middleware enabled: cors");
    }

    // 安全中间件（从配置文件读取）
    if config.middleware_enabled("security") {
        let security_config = config.security.clone().unwrap_or_default();
        security::apply_security_middleware_with_config(&mut app, &security_config);
    }

    app
}
