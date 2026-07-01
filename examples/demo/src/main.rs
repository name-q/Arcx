mod controller;
mod helper;
mod middleware;
mod prelude;
mod router;
mod service;

use arcx_core::prelude::*;
use crate::middleware::auth::JwtAuth;

#[tokio::main]
async fn main() {
    Arcx::new()
        .auth(JwtAuth::new("your-secret-key"))
        .routes(router::routes)
        .run()
        .await;
}
