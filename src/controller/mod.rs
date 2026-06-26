//! Controller 层
//! 约定：
//! - 每个文件对应一个资源（user.rs, article.rs, health.rs）
//! - 文件名即路由前缀: user.rs → /api/user
//! - 每个模块必须暴露 pub fn routes() -> Router<AppState>
//! - Controller 只做：取参数 → 调 Service → 返结果

pub mod health;
pub mod user;
pub mod echo_ws;
