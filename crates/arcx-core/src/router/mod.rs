//! 路由系统
//!
//! 提供约定式路由注册宏。

/// 约定式 Controller 注册宏
/// 模块名 = 路由前缀: user → /api/user
///
/// 用法：
/// ```rust
/// let public = arcx_core::register_controllers!(AppState, controller, user, health);
/// ```
#[macro_export]
macro_rules! register_controllers {
    ($state_type:ty, $base:ident, $( $module:ident ),* $(,)?) => {{
        let mut router: axum::Router<$state_type> = axum::Router::new();
        $(
            let prefix = concat!("/", stringify!($module));
            let sub_routes = $base::$module::routes();
            $crate::prelude::tracing::info!("  Controller loaded: {} → /api{}", stringify!($module), prefix);
            router = router.nest(prefix, sub_routes);
        )*
        router
    }};
}

/// 注册受保护路由（需要鉴权守卫的路由）
///
/// 用法：
/// ```rust
/// let protected = arcx_core::register_protected_controllers!(AppState, state, controller, user);
/// ```
#[macro_export]
macro_rules! register_protected_controllers {
    ($state_type:ty, $state:expr, $base:ident, $( $module:ident ),* $(,)?) => {{
        let mut router: axum::Router<$state_type> = axum::Router::new();
        $(
            let prefix = concat!("/", stringify!($module));
            let sub_routes = $base::$module::protected_routes();
            $crate::prelude::tracing::info!("  Protected routes: {} → /api{}", stringify!($module), prefix);
            router = router.nest(prefix, sub_routes);
        )*
        router.route_layer(axum::middleware::from_fn_with_state(
            $state.clone(),
            $crate::guard::auth_guard,
        ))
    }};
}
