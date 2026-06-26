pub mod user;

use sea_orm::{Database, DatabaseConnection, DbErr};

/// 初始化数据库连接
/// 通过配置文件中的 database.url 连接数据库
pub async fn init_db(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    tracing::info!("Database connected: {}", database_url);
    Ok(db)
}
