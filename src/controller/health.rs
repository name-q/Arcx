//! 健康检查 Controller
//! 路由前缀: /api/health

use axum::{routing::get, Json, Router};
use serde_json::json;

use crate::context::{AppState, Context};

/// 注册路由
/// 约定：每个 controller 暴露此函数
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(check))
        .route("/detail", get(detail))
}

/// GET /api/health
/// 基础健康检查
async fn check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok"
    }))
}

/// GET /api/health/detail
/// 详细健康信息（包含环境、版本）
async fn detail(ctx: Context) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "app": ctx.config.app.name,
        "version": ctx.config.app.version,
        "env": ctx.env(),
    }))
}
