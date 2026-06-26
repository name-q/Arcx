use axum::{middleware, routing::get, Router};

use crate::controller;
use crate::middleware::request_logger;

/// 构建应用路由
/// 约定：路由集中注册，按模块分组
pub fn build() -> Router {
    Router::new()
        .nest("/api", api_routes())
        .layer(middleware::from_fn(request_logger::request_logger))
}

/// API 路由组
fn api_routes() -> Router {
    Router::new()
        .route("/user", get(controller::user::list))
        .route("/user/:id", get(controller::user::info))
}
