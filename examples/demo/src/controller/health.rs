use axum::{routing::get, Json, Router};
use arcx_core::prelude::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(check))
}

async fn check(ctx: Context) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "app": ctx.config.app.name,
        "env": ctx.env(),
    }))
}
