//! CSRF 防护中间件
//!
//! 策略：
//! - 安全方法（GET/HEAD/OPTIONS）放行
//! - 非安全方法检查 `x-csrf-token` 或 `_csrf` 参数
//! - Token 来源：Cookie 中的 `_csrf_token` 值
//! - 支持配置豁免路径（如 webhook 回调）
//!
//! 流程：
//! 1. 首次请求 → 生成 csrf token → 写入 Cookie
//! 2. 前端表单/AJAX 带上 token（header 或 body）
//! 3. POST/PUT/DELETE 时校验 token 一致性

use axum::{
    extract::Request,
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::SecurityConfig;

/// CSRF Token Cookie 名
const CSRF_COOKIE_NAME: &str = "_csrf_token";
/// CSRF Token Header 名（前端 AJAX 用）
const CSRF_HEADER_NAME: &str = "x-csrf-token";

/// CSRF 防护中间件
pub async fn csrf_guard(
    config: SecurityConfig,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // 安全方法直接放行
    if is_safe_method(&method) {
        return next.run(request).await;
    }

    // 检查豁免路径
    if config.csrf_ignore.iter().any(|prefix| path.starts_with(prefix)) {
        return next.run(request).await;
    }

    // 非安全方法：校验 CSRF Token
    let cookie_token = extract_cookie_from_request(&request, CSRF_COOKIE_NAME);
    let request_token = extract_csrf_token_from_request(&request);

    match (cookie_token, request_token) {
        (Some(cookie_val), Some(req_val)) if constant_time_eq(cookie_val.as_bytes(), req_val.as_bytes()) => {
            // Token 匹配，放行
            next.run(request).await
        }
        _ => {
            tracing::warn!("CSRF token mismatch: {} {}", method, path);
            (
                StatusCode::FORBIDDEN,
                serde_json::json!({
                    "error": "CSRF token mismatch",
                    "message": "Missing or invalid CSRF token"
                }).to_string(),
            ).into_response()
        }
    }
}

/// 判断是否为安全方法
fn is_safe_method(method: &Method) -> bool {
    matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS | &Method::TRACE)
}

/// 从请求中提取 CSRF Token（优先 header，其次 query）
fn extract_csrf_token_from_request(request: &Request) -> Option<String> {
    // 优先从 header 获取
    if let Some(val) = request.headers().get(CSRF_HEADER_NAME) {
        return val.to_str().ok().map(|s| s.to_string());
    }

    // 从 query string 获取 _csrf 参数
    if let Some(query) = request.uri().query() {
        for pair in query.split('&') {
            if let Some(value) = pair.strip_prefix("_csrf=") {
                return Some(value.to_string());
            }
        }
    }

    None
}

/// 从请求 Cookie 头中提取指定 cookie
fn extract_cookie_from_request(request: &Request, name: &str) -> Option<String> {
    request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            for pair in cookies.split(';') {
                let pair = pair.trim();
                if let Some(value) = pair.strip_prefix(&format!("{}=", name)) {
                    return Some(value.to_string());
                }
            }
            None
        })
}

/// 生成 CSRF Token
#[allow(dead_code)]
pub fn generate_csrf_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:x}{:x}", ts, count)
}

/// 常量时间比较
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}
