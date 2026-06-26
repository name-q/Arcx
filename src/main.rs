mod config;
mod context;
mod controller;
mod error;
mod middleware;
mod router;

use context::AppState;
use tracing_subscriber;

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

    // 构建共享状态
    let state = AppState::new(cfg.clone());

    // 构建路由（自动加载所有 controller）
    let app = router::build(state);

    // 启动服务
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    tracing::info!("Arcx server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
