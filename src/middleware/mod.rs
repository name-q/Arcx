//! 中间件层
//! 约定：
//! - 全局中间件放在 src/middleware/ 目录
//! - 中间件分为两类：
//!   1. 框架内置中间件（request_logger, cors, error_handler）
//!   2. 用户自定义中间件（通过配置启用）
//! - 中间件执行顺序：外层先注册后执行（洋葱模型）
//!
//! 注意：axum 的 layer 是"后注册先执行"的
//! 所以注册顺序 = [cors, logger] 时，实际执行是 logger → cors → handler → cors → logger

pub mod cors;
pub mod request_logger;

use axum::{middleware, Router};
use crate::context::AppState;

/// 注册所有全局中间件
/// 按照洋葱模型组织：
/// 最外层: CORS（最先接触请求，最后处理响应）
/// 内层: 请求日志（记录实际处理时间）
///
/// 后续可通过配置文件控制中间件开关和顺序
pub fn apply_global_middleware(router: Router<AppState>) -> Router<AppState> {
    router
        .layer(middleware::from_fn(request_logger::request_logger))
        .layer(cors::cors_layer())
}
