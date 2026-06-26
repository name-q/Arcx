use axum::Router;

use crate::context::AppState;
use crate::controller;
use crate::middleware;

/// 约定式 Controller 注册宏
/// 模块名 = 路由前缀: user → /api/user
///
/// 用法：
/// ```rust
/// register_controllers!(
///     user,       // → /api/user
///     article,    // → /api/article
/// );
/// ```
macro_rules! register_controllers {
    ($( $module:ident ),* $(,)?) => {{
        let mut router: Router<AppState> = Router::new();
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
    // 1. 注册所有 controller（约定式加载）
    let api = register_controllers!(
        user,
        health,
    );

    // 2. 组装路由 + 中间件 + 状态
    let app = Router::new().nest("/api", api);

    // 3. 应用全局中间件
    middleware::apply_global_middleware(app).with_state(state)
}
