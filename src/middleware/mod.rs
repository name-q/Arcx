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
//! ```

pub mod cors;
pub mod request_logger;

use axum::{middleware, Router};
use crate::config::AppConfig;
use crate::context::AppState;

/// 注册全局中间件
/// 根据配置决定开启哪些中间件
pub fn apply_global_middleware(router: Router<AppState>, config: &AppConfig) -> Router<AppState> {
    let mut app = router;

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

    app
}
