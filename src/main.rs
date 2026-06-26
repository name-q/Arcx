mod config;
mod context;
mod controller;
mod error;
mod middleware;
mod model;
mod router;
mod service;

use context::AppState;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
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

    // 初始化数据库连接
    let db = model::init_db(&cfg.database.url)
        .await
        .expect("Failed to connect database");

    // 自动建表（开发环境使用，生产环境应使用 migration）
    init_tables(&db).await;

    // 构建共享状态
    let state = AppState::new(cfg.clone(), db);

    // 构建路由（传入状态）
    let app = router::build(state);

    // 启动服务
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    tracing::info!("Arcx server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// 开发环境自动建表
async fn init_tables(db: &sea_orm::DatabaseConnection) {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT ''
        )
    "#;
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql.to_string()))
        .await
        .expect("Failed to create tables");
    tracing::info!("Database tables initialized");
}
