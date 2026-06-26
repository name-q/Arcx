#![allow(dead_code)]
mod client;
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
mod schedule;
mod service;
mod ws;

use client::event_bus::EventBus;
use client::registry::ClientRegistry;
use config::watcher::ConfigWatcher;
use context::AppState;
use lifecycle::{Lifecycle, shutdown_channel};
use plugin::PluginManager;
use schedule::manager::ScheduleManager;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    // 1. 初始化日志
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S".to_string(),
        ))
        .init();

    // 2. 加载配置
    let cfg = config::AppConfig::load();
    tracing::info!("{} v{} starting...", cfg.app.name, cfg.app.version);
    tracing::info!("environment: {}", cfg.app.env);

    // 3. 创建事件总线（框架核心通信机制）
    let event_bus = EventBus::new(128);

    // 4. 配置热更新通道（watch channel）
    let (_config_notifier, config_watcher) = ConfigWatcher::new(cfg.clone());
    // _config_notifier 在需要热更新时使用（如文件监听、API 触发）

    // 5. 读取原始配置（传递给插件系统）
    let raw_config = config::load_raw_config();

    // 6. 初始化插件
    let mut plugin_manager = PluginManager::new();
    plugin_manager.auto_register_builtin(&raw_config);

    if let Err(e) = plugin_manager.init_all(&raw_config).await {
        tracing::error!("Plugin init failed: {}", e);
        std::process::exit(1);
    }

    // 7. 构建共享状态
    let resources = plugin_manager.take_resources();
    let state = AppState::with_resources(cfg.clone(), resources.clone(), event_bus.clone(), config_watcher.clone());

    // 8. 初始化客户端注册表
    let mut client_registry = ClientRegistry::new(event_bus.clone());
    // 用户自定义客户端在此注册:
    // client_registry.register(Arc::new(MyRedisClient::new()), Some(Duration::from_secs(10))).await?;
    client_registry.start_health_checks();

    // 9. 初始化定时任务系统
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

    // 10. 创建关闭信号通道
    let (shutdown_trigger, _shutdown_signal) = shutdown_channel();

    // 11. 构建生命周期管理器
    let mut lifecycle = Lifecycle::new(event_bus.clone());

    // 注册关闭钩子：插件关闭
    let pm_for_shutdown = plugin_manager;
    lifecycle.on_shutdown(move || async move {
        pm_for_shutdown.shutdown_all().await;
    });

    // 注册关闭钩子：客户端断开
    let mut cr_for_shutdown = client_registry;
    lifecycle.on_shutdown(move || async move {
        cr_for_shutdown.shutdown().await;
    });

    // 注册关闭钩子：定时任务停止
    let sm_for_shutdown = schedule_manager.clone();
    lifecycle.on_shutdown(move || async move {
        sm_for_shutdown.write().await.shutdown().await;
    });

    lifecycle.mark_started();

    // 12. 构建路由并启动服务
    let app = router::build(state);
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    tracing::info!("Arcx server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // 13. 启动服务，带优雅关闭
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            wait_for_shutdown_signal().await;
            shutdown_trigger.shutdown();
            // 给关闭信号一点传播时间
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        })
        .await
        .unwrap();

    // 14. 服务停止后执行关闭流程
    lifecycle.shutdown().await;
}

/// 注册定时任务
/// 约定：所有定时任务在此统一注册
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
