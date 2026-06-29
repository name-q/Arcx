use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

use crate::logger::trace_id::TraceId;

/// 请求日志中间件
/// 记录每个请求的方法、路径、状态码、耗时和 trace_id
pub async fn request_logger(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    // 获取 trace_id（由 trace_id_middleware 注入）
    let trace_id = request
        .extensions()
        .get::<TraceId>()
        .map(|t| t.0.clone())
        .unwrap_or_else(|| "-".to_string());

    tracing::info!("[{}] --> {} {}", trace_id, method, uri.path());

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    let level = if status.is_server_error() {
        tracing::Level::ERROR
    } else if status.is_client_error() {
        tracing::Level::WARN
    } else {
        tracing::Level::INFO
    };

    match level {
        tracing::Level::ERROR => tracing::error!(
            "[{}] <-- {} {} {} [{:.2}ms]",
            trace_id, method, uri.path(), status.as_u16(), duration.as_secs_f64() * 1000.0
        ),
        tracing::Level::WARN => tracing::warn!(
            "[{}] <-- {} {} {} [{:.2}ms]",
            trace_id, method, uri.path(), status.as_u16(), duration.as_secs_f64() * 1000.0
        ),
        _ => tracing::info!(
            "[{}] <-- {} {} {} [{:.2}ms]",
            trace_id, method, uri.path(), status.as_u16(), duration.as_secs_f64() * 1000.0
        ),
    }

    response
}
