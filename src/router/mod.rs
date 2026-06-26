use axum::{middleware, routing::get, Router};

use crate::context::AppState;
use crate::controller;
use crate::middleware::{cors, request_logger};

/// 构建应用路由
/// 约定：路由集中注册，按模块分组
pub fn build(state: AppState) -> Router {
    Router::new()
        .nest("/api", api_routes())
        .layer(middleware::from_fn(request_logger::request_logger))
        .layer(cors::cors_layer())
        .with_state(state)
}

/// API 路由组
fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/user", get(controller::user::list).post(controller::user::create))
        .route("/user/{id}", get(controller::user::info))
}
