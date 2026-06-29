//! Arcx Demo Application

mod controller;
mod helper;
mod router;

use arcx_core::prelude::*;

#[tokio::main]
async fn main() {
    Arcx::new()
        .routes(router::routes)
        .run()
        .await;
}
