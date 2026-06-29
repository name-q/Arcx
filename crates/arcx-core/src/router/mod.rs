//! 路由系统
//!
//! 提供完全自由的路由注册方式。
//!
//! ## 用户使用方式
//!
//! ```rust
//! // router.rs
//! use arcx_core::prelude::*;
//! use crate::controller;
//!
//! pub fn routes(r: &mut ArcxRouter) {
//!     r.get("/api/home", controller::home::index);
//!     r.get("/api/home/:id", controller::home::show);
//!     r.post("/api/home", controller::home::create);
//!
//!     // 需要鉴权的路由分组
//!     r.scope("/api/admin", |s| {
//!         s.guard(AuthGuard);
//!         s.get("/dashboard", controller::admin::dashboard);
//!     });
//! }
//! ```

use axum::{
    handler::Handler,
    routing::{self, MethodRouter},
    Router,
};

use crate::context::AppState;

/// RESTful 资源处理器集合（可选快捷方式）
///
/// 通过 builder 模式收集约定方法：
/// - index   → GET    /{prefix}
/// - show    → GET    /{prefix}/:id
/// - create  → POST   /{prefix}
/// - update  → PUT    /{prefix}/:id
/// - destroy → DELETE /{prefix}/:id
///
/// 注意：这是可选的便利写法，推荐直接用 r.get/post/put/delete 自由组合。
pub struct ResourceHandlers {
    pub index: Option<MethodRouter<AppState>>,
    pub show: Option<MethodRouter<AppState>>,
    pub create: Option<MethodRouter<AppState>>,
    pub update: Option<MethodRouter<AppState>>,
    pub destroy: Option<MethodRouter<AppState>>,
}

impl ResourceHandlers {
    pub fn new() -> Self {
        Self {
            index: None,
            show: None,
            create: None,
            update: None,
            destroy: None,
        }
    }

    /// 设置 index handler (GET /)
    pub fn index<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.index = Some(routing::get(handler));
        self
    }

    /// 设置 show handler (GET /:id)
    pub fn show<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.show = Some(routing::get(handler));
        self
    }

    /// 设置 create handler (POST /)
    pub fn create<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.create = Some(routing::post(handler));
        self
    }

    /// 设置 update handler (PUT /:id)
    pub fn update<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.update = Some(routing::put(handler));
        self
    }

    /// 设置 destroy handler (DELETE /:id)
    pub fn destroy<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.destroy = Some(routing::delete(handler));
        self
    }
}

impl Default for ResourceHandlers {
    fn default() -> Self {
        Self::new()
    }
}

/// Arcx 路由注册器
///
/// 在 `router.rs` 中通过此结构体自由声明所有路由。
///
/// ## 示例
///
/// ```rust
/// pub fn routes(r: &mut ArcxRouter) {
///     r.get("/api/home", controller::home::index);
///     r.get("/api/home/:id", controller::home::show);
///     r.post("/api/home", controller::home::create);
///     r.put("/api/home/:id", controller::home::update);
///     r.delete("/api/home/:id", controller::home::destroy);
///
///     // 鉴权分组
///     r.guarded_scope("/api/admin", |s| {
///         s.get("/dashboard", controller::admin::dashboard);
///     });
/// }
/// ```
pub struct ArcxRouter {
    router: Router<AppState>,
    guarded_router: Router<AppState>,
    has_guarded: bool,
}

impl ArcxRouter {
    pub(crate) fn new() -> Self {
        Self {
            router: Router::new(),
            guarded_router: Router::new(),
            has_guarded: false,
        }
    }

    // ─── 公开路由方法 ───────────────────────────

    /// GET 路由
    pub fn get<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.router = std::mem::take(&mut self.router).route(path, routing::get(handler));
        self
    }

    /// POST 路由
    pub fn post<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.router = std::mem::take(&mut self.router).route(path, routing::post(handler));
        self
    }

    /// PUT 路由
    pub fn put<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.router = std::mem::take(&mut self.router).route(path, routing::put(handler));
        self
    }

    /// DELETE 路由
    pub fn delete<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.router = std::mem::take(&mut self.router).route(path, routing::delete(handler));
        self
    }

    /// PATCH 路由
    pub fn patch<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.router = std::mem::take(&mut self.router).route(path, routing::patch(handler));
        self
    }

    // ─── 路由分组 ───────────────────────────

    /// 路由分组（共享前缀）
    pub fn scope(&mut self, prefix: &str, f: impl FnOnce(&mut ArcxRouter)) -> &mut Self {
        let mut sub = ArcxRouter::new();
        f(&mut sub);
        self.router = std::mem::take(&mut self.router).nest(prefix, sub.router);
        if sub.has_guarded {
            self.guarded_router =
                std::mem::take(&mut self.guarded_router).nest(prefix, sub.guarded_router);
            self.has_guarded = true;
        }
        self
    }

    /// 需要鉴权的路由分组（分组内所有路由自动加鉴权守卫）
    pub fn guarded_scope(&mut self, prefix: &str, f: impl FnOnce(&mut ArcxRouter)) -> &mut Self {
        let mut sub = ArcxRouter::new();
        f(&mut sub);
        // 把 sub 的公开路由合并到 guarded_router（因为整个 scope 都需要鉴权）
        self.guarded_router =
            std::mem::take(&mut self.guarded_router).nest(prefix, sub.router);
        self.has_guarded = true;
        self
    }

    // ─── RESTful 资源路由（可选快捷方式）───────────────────────────

    /// 注册 RESTful 资源路由（可选快捷方式）
    ///
    /// 根据 ResourceHandlers 中注册的方法自动映射路由：
    /// - index   → GET    {prefix}
    /// - show    → GET    {prefix}/:id
    /// - create  → POST   {prefix}
    /// - update  → PUT    {prefix}/:id
    /// - destroy → DELETE {prefix}/:id
    pub fn resources(&mut self, prefix: &str, handlers: ResourceHandlers) -> &mut Self {
        let resource_router = Self::build_resource_router(handlers);
        self.router = std::mem::take(&mut self.router).nest(prefix, resource_router);
        tracing::info!("  Resource: {}", prefix);
        self
    }

    /// 注册需要鉴权的 RESTful 资源路由
    pub fn guarded_resources(&mut self, prefix: &str, handlers: ResourceHandlers) -> &mut Self {
        let resource_router = Self::build_resource_router(handlers);
        self.guarded_router =
            std::mem::take(&mut self.guarded_router).nest(prefix, resource_router);
        self.has_guarded = true;
        tracing::info!("  Guarded Resource: {}", prefix);
        self
    }

    /// 直接合并一个 axum Router（escape hatch）
    pub fn merge_router(&mut self, router: Router<AppState>) -> &mut Self {
        self.router = std::mem::take(&mut self.router).merge(router);
        self
    }

    // ─── 内部方法 ───────────────────────────

    fn build_resource_router(handlers: ResourceHandlers) -> Router<AppState> {
        let mut router = Router::new();

        // 集合路由: /
        let mut collection: Option<MethodRouter<AppState>> = None;

        if let Some(index_handler) = handlers.index {
            collection = Some(index_handler);
        }

        if let Some(create_handler) = handlers.create {
            collection = Some(match collection {
                Some(existing) => existing.merge(create_handler),
                None => create_handler,
            });
        }

        if let Some(col) = collection {
            router = router.route("/", col);
        }

        // 单项路由: /:id
        let mut item: Option<MethodRouter<AppState>> = None;

        if let Some(show_handler) = handlers.show {
            item = Some(show_handler);
        }

        if let Some(update_handler) = handlers.update {
            item = Some(match item {
                Some(existing) => existing.merge(update_handler),
                None => update_handler,
            });
        }

        if let Some(destroy_handler) = handlers.destroy {
            item = Some(match item {
                Some(existing) => existing.merge(destroy_handler),
                None => destroy_handler,
            });
        }

        if let Some(it) = item {
            router = router.route("/:id", it);
        }

        router
    }

    /// 构建最终的 axum Router（框架内部调用）
    pub(crate) fn build(self, state: &AppState) -> Router<AppState> {
        let mut app = self.router;

        if self.has_guarded {
            let guarded_with_layer = self.guarded_router.route_layer(
                axum::middleware::from_fn_with_state(state.clone(), crate::guard::auth_guard),
            );
            app = app.merge(guarded_with_layer);
        }

        app
    }
}

impl Default for ArcxRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 旧宏（deprecated，向后兼容）───────────────────────────

/// 注册 controller 路由（已废弃，请使用 Arcx::new().routes() 方式）
#[macro_export]
#[deprecated(since = "0.1.1", note = "Use Arcx::new().routes(router::routes) instead")]
macro_rules! register_controllers {
    ($($controller:path),* $(,)?) => {};
}
