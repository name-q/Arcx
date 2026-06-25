mod controller;
mod service;
mod router;
mod config;

use tracing_subscriber;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置
    let cfg = config::AppConfig::load();

    // 构建路由
    let app = router::build();

    // 启动服务
    let addr = format!("{}:{}", cfg.host, cfg.port);
    tracing::info!("Arcx server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
