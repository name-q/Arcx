//! Arcx Demo Application
//! Demonstrates framework features: controllers, plugins, middleware, etc.

mod controller;

use arcx_core::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // 1. Load config
    let cfg = Arcx::load_config();

    // 2. Init logging
    arcx_core::logger::init(&cfg.logger);
    tracing::info!("{} v{} starting...", cfg.app.name, cfg.app.version);
    tracing::info!("environment: {}", cfg.app.env);

    // 3. Event bus & config watcher
    let event_bus = EventBus::new(128);
    let (_notifier, config_watcher) = ConfigWatcher::new(cfg.clone());

    // 4. Plugins
    let raw_config = Arcx::load_raw_config();
    let mut plugin_manager = PluginManager::new();
    plugin_manager.auto_register_builtin(&raw_config);
    if let Err(e) = plugin_manager.init_all(&raw_config).await {
        tracing::error!("Plugin init failed: {}", e);
        std::process::exit(1);
    }

    // 5. HTTP Client
    let http_client = HttpClient::new(cfg.httpclient.clone());

    // 6. Build shared state
    let mut resources = plugin_manager.take_resources();
    resources.insert(
        std::any::TypeId::of::<HttpClient>(),
        Arc::new(http_client),
    );
    let state = AppState::with_resources(
        cfg.clone(),
        resources,
        event_bus.clone(),
        config_watcher,
    );

    // 7. Build routes
    let public = arcx_core::register_controllers!(AppState, controller, home, health);
    let api = axum::Router::new().nest("/api", public);
    let app = apply_global_middleware(api, &cfg).with_state(state);

    // 8. Start server
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    tracing::info!("Arcx server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("Shutting down...");
        })
        .await
        .unwrap();

    plugin_manager.shutdown_all().await;
}
