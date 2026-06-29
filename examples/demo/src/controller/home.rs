use axum::{routing::get, Json, Router};
use arcx_core::prelude::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
}

async fn index(ctx: Context) -> Json<serde_json::Value> {
    Json(json!({
        "name": ctx.config.app.name,
        "version": ctx.config.app.version,
        "message": "Welcome to Arcx!"
    }))
}
