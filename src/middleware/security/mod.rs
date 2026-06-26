//! 安全中间件
//!
//! 企业级安全防护，开箱即用：
//! - CSRF 防护（token 校验）
//! - 安全响应头（XSS/Clickjacking/MIME sniffing 防护）
//! - HSTS（强制 HTTPS）
//!
//! 配置方式：
//! ```toml
//! [security]
//! csrf = true                    # CSRF 防护
//! xss_protection = true          # X-XSS-Protection
//! frame_deny = true              # X-Frame-Options: DENY
//! content_type_nosniff = true    # X-Content-Type-Options: nosniff
//! hsts = false                   # 严格传输安全（生产环境用）
//! hsts_max_age = 31536000        # HSTS 有效期（秒）
//! ```

pub mod csrf;
pub mod headers;

use axum::{middleware, Router};
use crate::context::AppState;

/// 安全配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SecurityConfig {
    /// CSRF 防护
    #[serde(default = "default_true")]
    pub csrf: bool,
    /// X-XSS-Protection 头
    #[serde(default = "default_true")]
    pub xss_protection: bool,
    /// X-Frame-Options: DENY
    #[serde(default = "default_true")]
    pub frame_deny: bool,
    /// X-Content-Type-Options: nosniff
    #[serde(default = "default_true")]
    pub content_type_nosniff: bool,
    /// Strict-Transport-Security (HSTS)
    #[serde(default)]
    pub hsts: bool,
    /// HSTS max-age（秒）
    #[serde(default = "default_hsts_max_age")]
    pub hsts_max_age: u64,
    /// CSRF 豁免路径前缀（如 webhook）
    #[serde(default)]
    pub csrf_ignore: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            csrf: true,
            xss_protection: true,
            frame_deny: true,
            content_type_nosniff: true,
            hsts: false,
            hsts_max_age: default_hsts_max_age(),
            csrf_ignore: vec![],
        }
    }
}

fn default_true() -> bool { true }
fn default_hsts_max_age() -> u64 { 31536000 }

/// 应用安全中间件（从配置读取）
pub fn apply_security_middleware_with_config(
    app: &mut Router<AppState>,
    config: &SecurityConfig,
) {
    // 安全响应头（始终启用）
    let headers_config = config.clone();
    *app = app.clone().layer(middleware::from_fn(move |req, next| {
        let cfg = headers_config.clone();
        headers::security_headers(cfg, req, next)
    }));
    tracing::info!("  Security: headers enabled");

    // CSRF 防护（配置决定）
    if config.csrf {
        let csrf_config = config.clone();
        *app = app.clone().layer(middleware::from_fn(move |req, next| {
            let cfg = csrf_config.clone();
            csrf::csrf_guard(cfg, req, next)
        }));
        tracing::info!("  Security: CSRF enabled");
    } else {
        tracing::info!("  Security: CSRF disabled (API mode)");
    }
}
