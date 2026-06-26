mod config;
mod controller;
mod error;
mod middleware;
mod router;
mod service;

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
    tracing::info!("debug mode: {}", cfg.app.debug);

    // 构建路由（传入配置）
    let app = router::build();

    // 启动服务
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    tracing::info!("Arcx server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
