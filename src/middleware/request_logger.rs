use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// 请求日志中间件
/// 记录每个请求的方法、路径、状态码和耗时
pub async fn request_logger(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    tracing::info!("--> {} {}", method, uri.path());

    // 执行后续中间件和 handler
    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    tracing::info!(
        "<-- {} {} {} [{:.2}ms]",
        method,
        uri.path(),
        status.as_u16(),
        duration.as_secs_f64() * 1000.0
    );

    response
}
