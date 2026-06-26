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

use context::AppState;
use plugin::PluginManager;
use schedule::manager::ScheduleManager;
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
    plugin_manager.auto_register_builtin(&raw_config);

    if let Err(e) = plugin_manager.init_all(&raw_config).await {
        tracing::error!("Plugin init failed: {}", e);
        std::process::exit(1);
    }

    // 构建共享状态
    let resources = plugin_manager.take_resources();
    let state = AppState::with_resources(cfg.clone(), resources.clone());

    // 初始化定时任务系统
    let schedule_enabled = raw_config
        .get("schedule")
        .and_then(|s| s.get("enable"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let schedule_manager = Arc::new(RwLock::new(ScheduleManager::new()));

    if schedule_enabled {
        let mut sm = schedule_manager.write().await;
        // 注册示例任务（用户自定义任务也在此注册）
        register_schedule_jobs(&mut sm);

        let resources_arc = Arc::new(resources);
        if let Err(e) = sm.start(Arc::new(cfg.clone()), resources_arc).await {
            tracing::error!("Schedule start failed: {}", e);
        }
    } else {
        tracing::info!("Schedule system disabled");
    }

    // 生命周期
    let pm = Arc::new(RwLock::new(plugin_manager));
    let lifecycle = lifecycle::Lifecycle::new(pm.clone());

    // 构建路由
    let app = router::build(state);

    // 启动服务
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    tracing::info!("Arcx server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    let sm_shutdown = schedule_manager.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(lifecycle, sm_shutdown))
        .await
        .unwrap();
}

/// 注册定时任务
/// 约定：所有定时任务在此统一注册
fn register_schedule_jobs(sm: &mut ScheduleManager) {
    // 注册自定义任务
    sm.register(service::demo_jobs::HealthCheckJob);
}

/// 监听关闭信号，执行优雅退出
async fn shutdown_signal(
    lifecycle: lifecycle::Lifecycle,
    schedule_manager: Arc<RwLock<ScheduleManager>>,
) {
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

    // 停止定时任务
    schedule_manager.write().await.shutdown().await;

    lifecycle.shutdown().await;
}
