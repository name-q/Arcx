//! 安全响应头中间件
//!
//! 自动为所有响应添加安全防护头：
//! - X-Content-Type-Options: nosniff（防止 MIME 嗅探）
//! - X-Frame-Options: DENY（防止点击劫持）
//! - X-XSS-Protection: 1; mode=block（浏览器 XSS 过滤）
//! - Strict-Transport-Security（强制 HTTPS）

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};

use super::SecurityConfig;

/// 安全响应头中间件
pub async fn security_headers(
    config: SecurityConfig,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // X-Content-Type-Options
    if config.content_type_nosniff {
        headers.insert(
            "x-content-type-options",
            HeaderValue::from_static("nosniff"),
        );
    }

    // X-Frame-Options
    if config.frame_deny {
        headers.insert(
            "x-frame-options",
            HeaderValue::from_static("DENY"),
        );
    }

    // X-XSS-Protection
    if config.xss_protection {
        headers.insert(
            "x-xss-protection",
            HeaderValue::from_static("1; mode=block"),
        );
    }

    // Strict-Transport-Security (HSTS)
    if config.hsts {
        let value = format!("max-age={}; includeSubDomains", config.hsts_max_age);
        if let Ok(val) = HeaderValue::from_str(&value) {
            headers.insert("strict-transport-security", val);
        }
    }

    // 额外安全头
    headers.insert(
        "x-download-options",
        HeaderValue::from_static("noopen"),
    );
    headers.insert(
        "x-dns-prefetch-control",
        HeaderValue::from_static("off"),
    );

    response
}
