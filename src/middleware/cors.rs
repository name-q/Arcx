use tower_http::cors::{Any, CorsLayer};

/// CORS 中间件
/// 开发环境允许所有来源，生产环境可通过配置限制
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}
