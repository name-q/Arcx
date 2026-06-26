mod config;
mod context;
mod controller;
mod error;
mod extract;
mod guard;
mod lifecycle;
mod middleware;
mod plugin;
mod router;
mod service;

use context::AppState;
use plugin::PluginManager;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S".to_string(),
        ))
        .init();

    // 加载配置
    let cfg = config::AppConfig::load();
    tracing::info!("{} v{} starting...", cfg.app.name, cfg.app.version);
    tracing::info!("environment: {}", cfg.app.env);

    // 读取原始配置（传递给插件系统）
    let raw_config = config::load_raw_config();

    // 初始化插件
    let mut plugin_manager = PluginManager::new();
    
    // 自动注册内置插件（根据配置 enable 字段）
    plugin_manager.auto_register_builtin(&raw_config);

    if let Err(e) = plugin_manager.init_all(&raw_config).await {
        tracing::error!("Plugin init failed: {}", e);
        std::process::exit(1);
    }

    // 构建共享状态
    let state = AppState::with_resources(cfg.clone(), plugin_manager.take_resources());

    // 生命周期
    let pm = Arc::new(RwLock::new(plugin_manager));
    let lifecycle = lifecycle::Lifecycle::new(pm.clone());

    // 构建路由
    let app = router::build(state);

    // 启动服务
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    tracing::info!("Arcx server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(lifecycle))
        .await
        .unwrap();
}

/// 监听关闭信号，执行优雅退出
async fn shutdown_signal(lifecycle: lifecycle::Lifecycle) {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to listen ctrl+c");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to listen SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    lifecycle.shutdown().await;
}
