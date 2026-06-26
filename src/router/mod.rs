use axum::{middleware, Router};

use crate::context::AppState;
use crate::controller;
use crate::middleware as mw;

/// 框架路由构建器
/// 核心约定：
/// 1. 每个 controller 模块暴露 routes() -> Router<AppState>
/// 2. controller 文件名即路由前缀: user.rs → /api/user
/// 3. 使用 controllers! 宏一行注册所有 controller

/// 声明式 controller 注册宏
/// 约定：模块名 = 路由前缀
/// 展开后自动 nest 到 /api/{name}
///
/// 用法：
/// ```rust
/// register_controllers!(state,
///     user,       // → /api/user
///     article,    // → /api/article
///     health,     // → /api/health
/// );
/// ```
macro_rules! register_controllers {
    ($state:expr, $( $module:ident ),* $(,)?) => {{
        let mut router = Router::new();
        $(
            let prefix = concat!("/", stringify!($module));
            let sub_routes = controller::$module::routes();
            tracing::info!("  Controller loaded: {} → /api{}", stringify!($module), prefix);
            router = router.nest(prefix, sub_routes);
        )*
        tracing::info!("All controllers loaded");
        router
    }};
}

/// 构建完整的应用路由
pub fn build(state: AppState) -> Router {
    // 注册所有 controller（约定式自动加载）
    let api = register_controllers!(state,
        user,
        health,
    );

    Router::new()
        .nest("/api", api)
        .layer(middleware::from_fn(mw::request_logger::request_logger))
        .layer(mw::cors::cors_layer())
        .with_state(state)
}
