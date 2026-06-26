use axum::{middleware as axum_mw, Router};

use crate::context::AppState;
use crate::controller;
use crate::guard::auth_guard;
use crate::middleware;
use crate::ws::WsRegistry;

/// 约定式 Controller 注册宏
/// 模块名 = 路由前缀: user → /api/user
macro_rules! register_controllers {
    ($( $module:ident ),* $(,)?) => {{
        let mut router: Router<AppState> = Router::new();
        $(
            let prefix = concat!("/", stringify!($module));
            let sub_routes = controller::$module::routes();
            tracing::info!("  Controller loaded: {} → /api{}", stringify!($module), prefix);
            router = router.nest(prefix, sub_routes);
        )*
        router
    }};
}

/// 注册受保护路由（需要鉴权守卫的路由）
macro_rules! register_protected_controllers {
    ($state:expr, $( $module:ident ),* $(,)?) => {{
        let mut router: Router<AppState> = Router::new();
        $(
            let prefix = concat!("/", stringify!($module));
            let sub_routes = controller::$module::protected_routes();
            tracing::info!("  Protected routes: {} → /api{}", stringify!($module), prefix);
            router = router.nest(prefix, sub_routes);
        )*
        router.route_layer(axum_mw::from_fn_with_state($state.clone(), auth_guard))
    }};
}

/// 注册 WebSocket handlers
fn register_ws_handlers() -> WsRegistry {
    let mut registry = WsRegistry::new();
    registry.register(controller::echo_ws::EchoWs);
    registry
}

/// 构建完整的应用路由
pub fn build(state: AppState) -> Router {
    // 1. 公开路由
    let public = register_controllers!(
        user,
        health,
    );

    // 2. 受保护路由
    let protected = register_protected_controllers!(
        state,
        user,
    );

    tracing::info!("All controllers loaded");

    // 3. 合并 API 路由
    let api = public.merge(protected);
    let app = Router::new().nest("/api", api);

    // 4. WebSocket 路由（不在 /api 前缀下）
    let ws_router = register_ws_handlers().into_router();
    let app = app.merge(ws_router);

    // 5. 应用全局中间件
    let config = state.config.as_ref().clone();
    middleware::apply_global_middleware(app, &config).with_state(state)
}
