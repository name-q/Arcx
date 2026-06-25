use axum::{routing::get, Router};

use crate::controller;

/// 构建应用路由
/// 约定：路由集中注册，按模块分组
pub fn build() -> Router {
    Router::new()
        .nest("/api", api_routes())
}

/// API 路由组
fn api_routes() -> Router {
    Router::new()
        .route("/user", get(controller::user::list))
        .route("/user/:id", get(controller::user::info))
}
