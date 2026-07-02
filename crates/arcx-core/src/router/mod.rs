//! 路由系统
//!
//! 提供完全自由的路由注册方式，支持路由级中间件。
//! 中间件和 Controller 共用同一个 Ctx。
//!
//! ## 用户使用方式
//!
//! ```rust,ignore
//! use arcx_core::prelude::*;
//! use crate::controller;
//! use crate::middleware;
//!
//! pub fn routes(r: &mut ArcxRouter) {
//!     // 全局中间件
//!     r.middleware(middleware::log::handle);
//!
//!     // 普通路由
//!     r.get("/api/home", controller::home::index);
//!
//!     // 路由级中间件
//!     r.get("/api/admin", controller::admin::dashboard)
//!         .middleware(middleware::auth::handle);
//!
//!     // 串联多个中间件
//!     r.post("/api/user", controller::user::create)
//!         .middleware(middleware::auth::handle)
//!         .middleware(middleware::permission::handle);
//!
//!     // 路由分组
//!     r.scope("/api/v2", |s| {
//!         s.middleware(middleware::auth::handle);
//!         s.get("/users", controller::user::list);
//!     });
//! }
//! ```
//!
//! ## 中间件签名
//!
//! ```rust,ignore
//! use arcx_core::prelude::*;
//!
//! pub async fn handle(ctx: Ctx, next: Next, parts: ReqParts) -> Response {
//!     let token = ctx.header("authorization").unwrap_or("");
//!     ctx.set(AuthedUser { ... });
//!     ctx.next(next, parts).await
//! }
//! ```

use axum::{
    extract::State,
    handler::Handler,
    middleware::{from_fn, from_fn_with_state, Next},
    routing::{self, MethodRouter},
    Router,
};
use std::future::Future;

use crate::context::AppState;
use crate::ctx::{Ctx, ReqParts};

// ─── 中间件存储 ───────────────────────────

/// 中间件条目：分为 Ctx 风格（需要 state 构造 Ctx）和 Raw 风格（直接操作 Request）
enum MiddlewareEntry {
    /// 新签名：async fn(Ctx, Next, ReqParts) -> Response
    Ctx(Box<dyn FnOnce(Router<AppState>, AppState) -> Router<AppState> + Send>),
    /// 旧签名：async fn(Request, Next) -> Response（兼容）
    Raw(Box<dyn FnOnce(Router<AppState>) -> Router<AppState> + Send>),
}

// ─── RouteBuilder — 路由级中间件链式构建 ───────────────────────────

/// 路由构建器 — 支持为单个路由附加中间件
pub struct RouteBuilder<'a> {
    router: &'a mut ArcxRouter,
    path: String,
    method_router: MethodRouter<AppState>,
    layers: Vec<MiddlewareEntry>,
}

impl<'a> RouteBuilder<'a> {
    /// 为此路由附加中间件
    ///
    /// ```rust,ignore
    /// r.get("/admin", handler)
    ///     .middleware(auth::handle);
    /// ```
    pub fn middleware<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(Ctx, Next, ReqParts) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = axum::response::Response> + Send + 'static,
    {
        self.layers.push(MiddlewareEntry::Ctx(Box::new(
            move |router, state| {
                router.layer(from_fn_with_state(
                    state,
                    move |State(app_state): State<AppState>,
                          req: axum::extract::Request,
                          next: Next| {
                        let f = f.clone();
                        async move {
                            let (ctx, req_parts) = Ctx::from_request(&app_state, req);
                            f(ctx, next, req_parts).await
                        }
                    },
                ))
            },
        )));
        self
    }

    /// 为此路由附加中间件（旧 Request 签名，兼容）
    pub fn middleware_raw<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(axum::extract::Request, Next) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = axum::response::Response> + Send + 'static,
    {
        self.layers
            .push(MiddlewareEntry::Raw(Box::new(move |router| {
                router.layer(from_fn(f))
            })));
        self
    }
}

impl<'a> Drop for RouteBuilder<'a> {
    fn drop(&mut self) {
        let path = std::mem::take(&mut self.path);
        let method_router =
            std::mem::replace(&mut self.method_router, routing::get(|| async { "" }));
        let layers = std::mem::take(&mut self.layers);

        if layers.is_empty() {
            self.router.router =
                std::mem::take(&mut self.router.router).route(&path, method_router);
        } else {
            let sub = Router::new().route(&path, method_router);
            self.router.pending_routes.push(PendingRoute {
                router: sub,
                layers,
            });
        }
    }
}

/// 待构建的路由（有中间件，需要在 build 时注入 state）
struct PendingRoute {
    router: Router<AppState>,
    layers: Vec<MiddlewareEntry>,
}

// ─── ResourceHandlers ───────────────────────────

/// RESTful 资源处理器集合（可选快捷方式）
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

    pub fn index<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.index = Some(routing::get(handler));
        self
    }

    pub fn show<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.show = Some(routing::get(handler));
        self
    }

    pub fn create<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.create = Some(routing::post(handler));
        self
    }

    pub fn update<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        self.update = Some(routing::put(handler));
        self
    }

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

// ─── ArcxRouter ───────────────────────────

/// Arcx 路由注册器
pub struct ArcxRouter {
    router: Router<AppState>,
    guarded_router: Router<AppState>,
    has_guarded: bool,
    global_layers: Vec<MiddlewareEntry>,
    pending_routes: Vec<PendingRoute>,
    scope_entries: Vec<ScopeEntry>,
    guarded_scope_entries: Vec<ScopeEntry>,
}

impl ArcxRouter {
    pub(crate) fn new() -> Self {
        Self {
            router: Router::new(),
            guarded_router: Router::new(),
            has_guarded: false,
            global_layers: Vec::new(),
            pending_routes: Vec::new(),
            scope_entries: Vec::new(),
            guarded_scope_entries: Vec::new(),
        }
    }

    // ─── 全局中间件 ───────────────────────────

    /// 注册全局中间件（Ctx 风格）
    ///
    /// ```rust,ignore
    /// r.middleware(middleware::auth::handle);
    /// ```
    pub fn middleware<F, Fut>(&mut self, f: F) -> &mut Self
    where
        F: Fn(Ctx, Next, ReqParts) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = axum::response::Response> + Send + 'static,
    {
        self.global_layers.push(MiddlewareEntry::Ctx(Box::new(
            move |router, state| {
                router.layer(from_fn_with_state(
                    state,
                    move |State(app_state): State<AppState>,
                          req: axum::extract::Request,
                          next: Next| {
                        let f = f.clone();
                        async move {
                            let (ctx, req_parts) = Ctx::from_request(&app_state, req);
                            f(ctx, next, req_parts).await
                        }
                    },
                ))
            },
        )));
        self
    }

    /// 注册全局中间件（旧 Request 签名，兼容）
    pub fn middleware_raw<F, Fut>(&mut self, f: F) -> &mut Self
    where
        F: Fn(axum::extract::Request, Next) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = axum::response::Response> + Send + 'static,
    {
        self.global_layers
            .push(MiddlewareEntry::Raw(Box::new(move |router| {
                router.layer(from_fn(f))
            })));
        self
    }

    // ─── 路由方法 ───────────────────────────

    /// GET 路由
    pub fn get<H, T>(&mut self, path: &str, handler: H) -> RouteBuilder<'_>
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        RouteBuilder {
            router: self,
            path: path.to_string(),
            method_router: routing::get(handler),
            layers: Vec::new(),
        }
    }

    /// POST 路由
    pub fn post<H, T>(&mut self, path: &str, handler: H) -> RouteBuilder<'_>
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        RouteBuilder {
            router: self,
            path: path.to_string(),
            method_router: routing::post(handler),
            layers: Vec::new(),
        }
    }

    /// PUT 路由
    pub fn put<H, T>(&mut self, path: &str, handler: H) -> RouteBuilder<'_>
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        RouteBuilder {
            router: self,
            path: path.to_string(),
            method_router: routing::put(handler),
            layers: Vec::new(),
        }
    }

    /// DELETE 路由
    pub fn delete<H, T>(&mut self, path: &str, handler: H) -> RouteBuilder<'_>
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        RouteBuilder {
            router: self,
            path: path.to_string(),
            method_router: routing::delete(handler),
            layers: Vec::new(),
        }
    }

    /// PATCH 路由
    pub fn patch<H, T>(&mut self, path: &str, handler: H) -> RouteBuilder<'_>
    where
        H: Handler<T, AppState> + Clone + Send + 'static,
        T: 'static,
    {
        RouteBuilder {
            router: self,
            path: path.to_string(),
            method_router: routing::patch(handler),
            layers: Vec::new(),
        }
    }

    // ─── 路由分组 ───────────────────────────

    /// 路由分组（共享前缀）
    pub fn scope(&mut self, prefix: &str, f: impl FnOnce(&mut ArcxRouter)) -> &mut Self {
        let mut sub = ArcxRouter::new();
        f(&mut sub);
        self.scope_entries.push(ScopeEntry {
            prefix: prefix.to_string(),
            sub_router: sub,
        });
        self
    }

    /// 需要鉴权的路由分组
    pub fn guarded_scope(&mut self, prefix: &str, f: impl FnOnce(&mut ArcxRouter)) -> &mut Self {
        let mut sub = ArcxRouter::new();
        f(&mut sub);
        self.guarded_scope_entries.push(ScopeEntry {
            prefix: prefix.to_string(),
            sub_router: sub,
        });
        self.has_guarded = true;
        self
    }

    // ─── RESTful 资源路由 ───────────────────────────

    /// 注册 RESTful 资源路由
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

        let mut collection: Option<MethodRouter<AppState>> = None;
        if let Some(h) = handlers.index {
            collection = Some(h);
        }
        if let Some(h) = handlers.create {
            collection = Some(match collection {
                Some(existing) => existing.merge(h),
                None => h,
            });
        }
        if let Some(col) = collection {
            router = router.route("/", col);
        }

        let mut item: Option<MethodRouter<AppState>> = None;
        if let Some(h) = handlers.show {
            item = Some(h);
        }
        if let Some(h) = handlers.update {
            item = Some(match item {
                Some(existing) => existing.merge(h),
                None => h,
            });
        }
        if let Some(h) = handlers.destroy {
            item = Some(match item {
                Some(existing) => existing.merge(h),
                None => h,
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

        // 合并 pending_routes（有路由级中间件的路由）
        for pending in self.pending_routes {
            let mut sub = pending.router;
            for entry in pending.layers.into_iter().rev() {
                sub = match entry {
                    MiddlewareEntry::Ctx(apply) => apply(sub, state.clone()),
                    MiddlewareEntry::Raw(apply) => apply(sub),
                };
            }
            app = app.merge(sub);
        }

        // 合并 scope entries（递归 build）
        for entry in self.scope_entries {
            let sub_built = entry.sub_router.build(state);
            app = app.nest(&entry.prefix, sub_built);
        }

        // 合并 guarded
        if self.has_guarded {
            let mut guarded = self.guarded_router;
            for entry in self.guarded_scope_entries {
                let sub_built = entry.sub_router.build(state);
                guarded = guarded.nest(&entry.prefix, sub_built);
            }
            let guarded_with_layer = guarded.route_layer(from_fn_with_state(
                state.clone(),
                crate::guard::auth_guard,
            ));
            app = app.merge(guarded_with_layer);
        }

        // 应用全局中间件（反转以保证声明顺序 = 执行顺序）
        for entry in self.global_layers.into_iter().rev() {
            app = match entry {
                MiddlewareEntry::Ctx(apply) => apply(app, state.clone()),
                MiddlewareEntry::Raw(apply) => apply(app),
            };
        }

        app
    }
}

/// Scope 条目
struct ScopeEntry {
    prefix: String,
    sub_router: ArcxRouter,
}

impl Default for ArcxRouter {
    fn default() -> Self {
        Self::new()
    }
}
