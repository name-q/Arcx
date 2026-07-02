use axum::http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

use crate::config::CorsConfig;

/// 根据配置构建 CORS 中间件
pub fn cors_layer_from_config(config: &CorsConfig) -> CorsLayer {
    let mut layer = CorsLayer::new();

    // Origins
    if config.allowed_origins.is_empty() || config.allowed_origins.contains(&"*".to_string()) {
        layer = layer.allow_origin(Any);
    } else {
        let origins: Vec<HeaderValue> = config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse::<HeaderValue>().ok())
            .collect();
        layer = layer.allow_origin(origins);
    }

    // Methods
    if config.allowed_methods.is_empty() {
        layer = layer.allow_methods(Any);
    } else {
        let methods: Vec<Method> = config
            .allowed_methods
            .iter()
            .filter_map(|m| m.parse::<Method>().ok())
            .collect();
        layer = layer.allow_methods(methods);
    }

    // Headers
    if config.allowed_headers.is_empty() {
        layer = layer.allow_headers(Any);
    } else {
        let headers: Vec<axum::http::HeaderName> = config
            .allowed_headers
            .iter()
            .filter_map(|h| h.parse().ok())
            .collect();
        layer = layer.allow_headers(headers);
    }

    // Credentials
    if config.allow_credentials {
        layer = layer.allow_credentials(true);
    }

    // Max age
    if let Some(max_age) = config.max_age {
        layer = layer.max_age(std::time::Duration::from_secs(max_age));
    }

    layer
}
