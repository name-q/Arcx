#![allow(dead_code)]
mod client;
mod config;
mod context;
mod controller;
mod error;
mod extract;
mod guard;
mod httpclient;
mod lifecycle;
mod logger;
mod middleware;
mod plugin;
mod router;
mod schedule;
mod service;
mod session;
mod ws;

use client::event_bus::EventBus;
use client::registry::ClientRegistry;
use config::watcher::ConfigWatcher;
use context::AppState;
use httpclient::HttpClient;
use lifecycle::{Lifecycle, shutdown_channel};
use plugin::PluginManager;
use schedule::manager::ScheduleManager;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    // 1. 加载配置（日志初始化前需要先拿到 logger 配置）
    let cfg = config::AppConfig::load();

    // 2. 初始化日志系统（必须在最前面）
    logger::init(&cfg.logger);

    tracing::info!("{} v{} starting...", cfg.app.name, cfg.app.version);
    tracing::info!("environment: {}", cfg.app.env);

    // 3. 创建事件总线（框架核心通信机制）
    let event_bus = EventBus::new(128);

    // 4. 配置热更新通道（watch channel）
    let (_config_notifier, config_watcher) = ConfigWatcher::new(cfg.clone());

    // 5. 读取原始配置（传递给插件系统）
    let raw_config = config::load_raw_config();

    // 6. 初始化插件
    let mut plugin_manager = PluginManager::new();
    plugin_manager.auto_register_builtin(&raw_config);

    if let Err(e) = plugin_manager.init_all(&raw_config).await {
        tracing::error!("Plugin init failed: {}", e);
        std::process::exit(1);
    }

    // 7. 构建 HTTP 客户端并注入资源
    let http_client = HttpClient::new(cfg.httpclient.clone());
    tracing::info!("HttpClient initialized (timeout={}s, retries={})", cfg.httpclient.timeout, cfg.httpclient.max_retries);

    // 8. 构建共享状态
    let mut resources = plugin_manager.take_resources();
    // 注入 HttpClient 到资源池
    resources.insert(
        std::any::TypeId::of::<HttpClient>(),
        Arc::new(http_client),
    );
    // 注入 SessionConfig 到资源池（如果配置了）
    if let Some(ref session_config) = cfg.session {
        resources.insert(
            std::any::TypeId::of::<session::SessionConfig>(),
            Arc::new(session_config.clone()),
        );
    }

    let state = AppState::with_resources(cfg.clone(), resources.clone(), event_bus.clone(), config_watcher.clone());

    // 9. 初始化客户端注册表
    let mut client_registry = ClientRegistry::new(event_bus.clone());
    client_registry.start_health_checks();

    // 10. 初始化定时任务系统
    let schedule_enabled = raw_config
        .get("schedule")
        .and_then(|s| s.get("enable"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let schedule_manager = Arc::new(RwLock::new(ScheduleManager::new()));

    if schedule_enabled {
        let mut sm = schedule_manager.write().await;
        register_schedule_jobs(&mut sm);

        let resources_arc = Arc::new(resources);
        if let Err(e) = sm.start(Arc::new(cfg.clone()), resources_arc).await {
            tracing::error!("Schedule start failed: {}", e);
        }
    } else {
        tracing::info!("Schedule system disabled");
    }

    // 11. 创建关闭信号通道
    let (shutdown_trigger, _shutdown_signal) = shutdown_channel();

    // 12. 构建生命周期管理器
    let mut lifecycle = Lifecycle::new(event_bus.clone());

    let pm_for_shutdown = plugin_manager;
    lifecycle.on_shutdown(move || async move {
        pm_for_shutdown.shutdown_all().await;
    });

    let mut cr_for_shutdown = client_registry;
    lifecycle.on_shutdown(move || async move {
        cr_for_shutdown.shutdown().await;
    });

    let sm_for_shutdown = schedule_manager.clone();
    lifecycle.on_shutdown(move || async move {
        sm_for_shutdown.write().await.shutdown().await;
    });

    lifecycle.mark_started();

    // 13. 构建路由并启动服务
    let app = router::build(state);
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    tracing::info!("Arcx server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // 14. 启动服务，带优雅关闭
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            wait_for_shutdown_signal().await;
            shutdown_trigger.shutdown();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        })
        .await
        .unwrap();

    // 15. 服务停止后执行关闭流程
    lifecycle.shutdown().await;
}

/// 注册定时任务
fn register_schedule_jobs(sm: &mut ScheduleManager) {
    sm.register(service::demo_jobs::HealthCheckJob);
}

/// 监听系统信号
async fn wait_for_shutdown_signal() {
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
}
