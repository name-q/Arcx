//! 请求 Trace ID
//!
//! 每个请求自动生成唯一 trace_id，贯穿整个请求生命周期。
//! - 中间件自动注入到请求 extensions
//! - Context 可直接获取
//! - 日志自动带上 trace_id
//! - 响应头返回 X-Trace-Id

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 全局递增计数器
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// 请求 Trace ID 类型
#[derive(Debug, Clone)]
pub struct TraceId(pub String);

impl TraceId {
    /// 生成新的 trace_id
    /// 格式: {timestamp_ms_hex}-{counter_hex}
    /// 轻量级、有序、无需 uuid 依赖
    pub fn generate() -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("{:x}-{:04x}", ts, count & 0xFFFF))
    }
}

/// Trace ID 中间件
/// 自动为每个请求注入 TraceId，并在响应头返回
pub async fn trace_id_middleware(mut request: Request, next: Next) -> Response {
    // 优先从请求头获取（支持链路追踪透传）
    let trace_id = request
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| TraceId(s.to_string()))
        .unwrap_or_else(TraceId::generate);

    // 注入到 request extensions
    request.extensions_mut().insert(trace_id.clone());

    // 执行后续处理
    let mut response = next.run(request).await;

    // 响应头带上 trace_id
    if let Ok(val) = HeaderValue::from_str(&trace_id.0) {
        response.headers_mut().insert("x-trace-id", val);
    }

    response
}
