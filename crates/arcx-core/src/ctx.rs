//! Ctx — 请求作用域的服务定位器
//!
//! 核心能力：
//! - 请求元信息：ctx.header()、ctx.method()、ctx.path()、ctx.uri()、ctx.ip()
//! - 查询参数：ctx.query::<T>()
//! - 配置读取：ctx.conf("redis.url")、ctx.env()
//! - 插件获取：ctx.plugin::<T>()
//! - 服务定位：ctx.service::<T>()（同请求内懒创建+缓存）
//! - 状态读写：ctx.set(val)、ctx.get::<T>()（请求级数据共享）
//!
//! ## 统一使用
//!
//! 中间件和 Controller 使用同一个 Ctx，同样的 API：
//!
//! ```rust,ignore
//! // 中间件 — 有 next 和 req_parts
//! pub async fn auth(ctx: Ctx, next: Next, req_parts: ReqParts) -> Response {
//!     let token = ctx.header("authorization").unwrap_or("");
//!     ctx.set(AuthedUser { user_id: 1, username: "test".into() });
//!     ctx.next(next, req_parts).await
//! }
//!
//! // Controller — 无 next
//! pub async fn index(ctx: Ctx) -> AppResult<impl IntoResponse> {
//!     let user = ctx.get::<AuthedUser>().unwrap();
//!     Ok(Json(user))
//! }
//! ```
//!
//! ## Service 互调
//!
//! Service 持有 Ctx（Arc 内部，clone 零成本），通过 ctx.service::<T>() 互调：
//! ```rust,ignore
//! impl UserService {
//!     pub async fn find_with_orders(&self, id: i64) -> AppResult<UserWithOrders> {
//!         let user = self.find(id).await?;
//!         let orders = self.ctx.service::<OrderService>().find_by_user(id).await?;
//!         Ok(UserWithOrders { user, orders })
//!     }
//! }
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};

use axum::extract::{ConnectInfo, FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{Extensions, HeaderMap, Method, Uri};
use axum::middleware::Next;
use axum::response::Response;

use crate::config::{AppConfig, FromTomlValue};
use crate::context::AppState;
use crate::error::{AppError, AppResult};
use crate::plugin::Plugin;

/// Service trait — 用户 Service 必须实现
///
/// 框架通过此 trait 懒创建 Service 实例并缓存。
pub trait Service: Send + Sync + 'static {
    /// 创建 Service 实例
    ///
    /// 每个请求内首次调用时执行，后续复用缓存。
    fn create(ctx: &Ctx) -> Arc<Self>
    where
        Self: Sized;
}

/// 请求作用域的服务定位器 + 状态容器
///
/// 中间件和 Controller 共享同一个类型。
/// 唯一区别：中间件多一个 `next` 参数，通过 `ctx.next(next, req_parts)` 放行。
#[derive(Clone)]
pub struct Ctx {
    /// App 级共享（所有请求共用）
    app: Arc<AppInner>,
    /// 请求级数据（Service 缓存 + 用户状态）
    request: Arc<RequestInner>,
}

/// App 级共享数据
struct AppInner {
    config: Arc<AppConfig>,
    resources: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

/// 请求级数据
struct RequestInner {
    /// 请求方法
    method: Method,
    /// 请求 URI（包含 path + query string）
    uri: Uri,
    /// 请求头
    headers: HeaderMap,
    /// 客户端 IP
    ip: Option<IpAddr>,
    /// Service 实例缓存（同一请求内复用）
    services: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    /// 请求级状态（ctx.set / ctx.get 读写）
    state: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    /// Extensions（上游中间件透传的底层存储）
    extensions: RwLock<Extensions>,
}

impl Ctx {
    /// 从 AppState + Parts 创建（Controller 的 FromRequestParts 用）
    pub(crate) fn from_parts(app_state: &AppState, parts: &Parts) -> Self {
        let ip = Self::extract_ip(parts);
        Self {
            app: Arc::new(AppInner {
                config: app_state.config.clone(),
                resources: app_state.resources_ref().clone(),
            }),
            request: Arc::new(RequestInner {
                method: parts.method.clone(),
                uri: parts.uri.clone(),
                headers: parts.headers.clone(),
                ip,
                services: RwLock::new(HashMap::new()),
                state: RwLock::new(HashMap::new()),
                extensions: RwLock::new(parts.extensions.clone()),
            }),
        }
    }

    /// 从 AppState + Request 创建（中间件包装层用）
    ///
    /// 拆解 Request，返回 Ctx + ReqParts（用于 ctx.next() 时重建 Request）
    pub(crate) fn from_request(
        app_state: &AppState,
        req: axum::extract::Request,
    ) -> (Self, ReqParts) {
        let (parts, body) = req.into_parts();
        let ip = Self::extract_ip(&parts);
        let ctx = Self {
            app: Arc::new(AppInner {
                config: app_state.config.clone(),
                resources: app_state.resources_ref().clone(),
            }),
            request: Arc::new(RequestInner {
                method: parts.method.clone(),
                uri: parts.uri.clone(),
                headers: parts.headers.clone(),
                ip,
                services: RwLock::new(HashMap::new()),
                state: RwLock::new(HashMap::new()),
                extensions: RwLock::new(parts.extensions),
            }),
        };
        let req_parts = ReqParts {
            method: parts.method,
            uri: parts.uri,
            version: parts.version,
            headers: parts.headers,
            body,
        };
        (ctx, req_parts)
    }

    /// 提取客户端 IP
    fn extract_ip(parts: &Parts) -> Option<IpAddr> {
        // 1. X-Forwarded-For（取第一个）
        if let Some(forwarded) = parts.headers.get("x-forwarded-for") {
            if let Ok(s) = forwarded.to_str() {
                if let Some(first) = s.split(',').next() {
                    if let Ok(ip) = first.trim().parse::<IpAddr>() {
                        return Some(ip);
                    }
                }
            }
        }

        // 2. X-Real-IP
        if let Some(real_ip) = parts.headers.get("x-real-ip") {
            if let Ok(s) = real_ip.to_str() {
                if let Ok(ip) = s.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }

        // 3. ConnectInfo（直连 IP）
        parts
            .extensions
            .get::<ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip())
    }

    // ─── 中间件放行 ─────────────────────────────────

    /// 放行请求到下一层（仅中间件使用）
    ///
    /// 会将 ctx.set() 写入的状态同步到 Request extensions，
    /// 确保下游中间件和 Controller 都能通过 ctx.get::<T>() 取到。
    ///
    /// ```rust,ignore
    /// pub async fn auth(ctx: Ctx, next: Next, req_parts: ReqParts) -> Response {
    ///     ctx.set(AuthedUser { id: 1, name: "test".into() });
    ///     ctx.next(next, req_parts).await
    /// }
    /// ```
    pub async fn next(self, next: Next, req_parts: ReqParts) -> Response {
        let req = self.rebuild_request(req_parts);
        next.run(req).await
    }

    /// 将 ctx 状态同步回 Request
    fn rebuild_request(&self, req_parts: ReqParts) -> axum::extract::Request {
        let mut extensions = {
            let ext = self.request.extensions.read().unwrap();
            ext.clone()
        };

        // 把 ctx.set() 写入的 state 同步到 extensions（用 CtxStateCarrier 包装）
        {
            let state = self.request.state.read().unwrap();
            if !state.is_empty() {
                // 合并到已有的 carrier（如果上游中间件也 set 过）
                let mut carrier_map: HashMap<TypeId, Arc<dyn Any + Send + Sync>> =
                    if let Some(existing) = extensions.get::<CtxStateCarrier>() {
                        existing.0.clone()
                    } else {
                        HashMap::new()
                    };
                for (k, v) in state.iter() {
                    carrier_map.insert(*k, v.clone());
                }
                extensions.insert(CtxStateCarrier(carrier_map));
            }
        }

        let mut builder = axum::http::Request::builder()
            .method(req_parts.method)
            .uri(req_parts.uri)
            .version(req_parts.version);

        *builder.headers_mut().unwrap() = req_parts.headers;
        let mut req = builder.body(req_parts.body).unwrap();
        *req.extensions_mut() = extensions;
        req
    }

    // ─── 请求元信息 ─────────────────────────────────

    /// 获取请求头值
    ///
    /// ```rust,ignore
    /// let token = ctx.header("authorization");
    /// let content_type = ctx.header("content-type");
    /// ```
    pub fn header(&self, key: &str) -> Option<&str> {
        self.request
            .headers
            .get(key)
            .and_then(|v| v.to_str().ok())
    }

    /// 获取请求方法
    pub fn method(&self) -> &Method {
        &self.request.method
    }

    /// 获取请求路径（不含 query string）
    pub fn path(&self) -> &str {
        self.request.uri.path()
    }

    /// 获取完整 URI（含 query string）
    pub fn uri(&self) -> &Uri {
        &self.request.uri
    }

    /// 获取客户端 IP
    ///
    /// 优先级：X-Forwarded-For > X-Real-IP > ConnectInfo
    pub fn ip(&self) -> Option<IpAddr> {
        self.request.ip
    }

    /// 反序列化 query string 为指定类型
    pub fn query<T: serde::de::DeserializeOwned>(&self) -> AppResult<T> {
        let query_str = self.request.uri.query().unwrap_or("");
        serde_urlencoded::from_str::<T>(query_str).map_err(|e| {
            AppError::bad_request(format!("Invalid query parameters: {}", e))
        })
    }

    // ─── 配置 ─────────────────────────────────

    /// 获取当前环境名
    pub fn env(&self) -> &str {
        &self.app.config.app.env
    }

    /// 是否为开发环境
    pub fn is_dev(&self) -> bool {
        self.app.config.app.env == "dev"
    }

    /// 通过 dot-notation 路径获取配置值
    ///
    /// ```rust,ignore
    /// let url: Option<String> = ctx.conf("redis.url");
    /// let port: Option<u16> = ctx.conf("server.port");
    /// ```
    pub fn conf<T: FromTomlValue>(&self, path: &str) -> Option<T> {
        self.app.config.get(path)
    }

    /// 将配置段反序列化为自定义结构
    pub fn conf_as<T: serde::de::DeserializeOwned>(&self, path: &str) -> Option<T> {
        self.app.config.get_as(path)
    }

    /// 获取强类型配置引用
    pub fn config(&self) -> &AppConfig {
        &self.app.config
    }

    // ─── 插件 ─────────────────────────────────

    /// 获取插件实例
    pub fn plugin<T: Plugin + 'static>(&self) -> AppResult<Arc<T>> {
        self.app
            .resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.clone().downcast::<T>().ok())
            .ok_or_else(|| {
                AppError::internal(format!(
                    "Plugin {} not registered",
                    std::any::type_name::<T>()
                ))
            })
    }

    /// 获取插件资源（通用版本，按类型取）
    pub fn resource<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.app
            .resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.clone().downcast::<T>().ok())
    }

    // ─── Service 定位 ─────────────────────────────────

    /// 获取 Service 实例
    ///
    /// 同一请求内首次调用 → 创建并缓存；后续调用 → 直接返回。
    pub fn service<T: Service>(&self) -> Arc<T> {
        let type_id = TypeId::of::<T>();

        // 快路径：读缓存
        {
            let cache = self.request.services.read().unwrap();
            if let Some(svc) = cache.get(&type_id) {
                return svc.clone().downcast::<T>().unwrap();
            }
        }

        // 慢路径：创建并写入缓存
        let svc = T::create(self);
        {
            let mut cache = self.request.services.write().unwrap();
            cache.insert(type_id, svc.clone());
        }
        svc
    }

    // ─── 请求级状态（set/get）─────────────────────────────────

    /// 写入请求级状态
    ///
    /// 中间件写入，下游中间件 / Controller 通过 get::<T>() 读取。
    ///
    /// ```rust,ignore
    /// ctx.set(AuthedUser { user_id: 1, username: "test".into() });
    /// ```
    pub fn set<T: Send + Sync + 'static>(&self, val: T) {
        let mut state = self.request.state.write().unwrap();
        state.insert(TypeId::of::<T>(), Arc::new(val));
    }

    /// 读取请求级状态
    ///
    /// 优先从当前 ctx.set() 写入的状态中查找，
    /// 再从上游中间件通过 ctx.set() + ctx.next() 传递下来的数据中查找。
    ///
    /// ```rust,ignore
    /// let user = ctx.get::<AuthedUser>();  // Option<Arc<T>>
    /// ```
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        // 1. 先查当前 ctx.set() 写入的
        {
            let state = self.request.state.read().unwrap();
            if let Some(v) = state
                .get(&TypeId::of::<T>())
                .and_then(|v| v.clone().downcast::<T>().ok())
            {
                return Some(v);
            }
        }
        // 2. 再查 extensions 中的 CtxStateCarrier（上游中间件 ctx.next() 传递的）
        {
            let ext = self.request.extensions.read().unwrap();
            if let Some(carrier) = ext.get::<CtxStateCarrier>() {
                if let Some(v) = carrier
                    .0
                    .get(&TypeId::of::<T>())
                    .and_then(|v| v.clone().downcast::<T>().ok())
                {
                    return Some(v);
                }
            }
        }
        None
    }

    // ─── 兼容旧 API ─────────────────────────────────

    /// 旧版状态获取（已废弃，请用 get::<T>()）
    #[deprecated(since = "0.1.7", note = "Use `get::<T>()` instead of `state::<T>()`")]
    pub fn state<T: Send + Sync + Clone + 'static>(&self) -> Option<Arc<T>> {
        self.get::<T>()
    }
}

/// Ctx 状态的载体 — 通过 extensions 在中间件间传递 ctx.set() 的值
#[derive(Clone)]
pub(crate) struct CtxStateCarrier(HashMap<TypeId, Arc<dyn Any + Send + Sync>>);

/// 中间件内 ctx.next() 需要的 Request body 和元数据
///
/// 中间件签名：`async fn(ctx: Ctx, next: Next, req_parts: ReqParts) -> Response`
///
/// Ctx 构造时消费了 Request 的头部信息，body 保存在 ReqParts 中。
/// 调用 ctx.next(next, req_parts) 时重建完整 Request 传给下一层。
pub struct ReqParts {
    method: Method,
    uri: Uri,
    version: axum::http::Version,
    headers: HeaderMap,
    body: axum::body::Body,
}

/// 实现 FromRequestParts，让 Ctx 能作为 Controller handler 参数自动提取
#[axum::async_trait]
impl<S> FromRequestParts<S> for Ctx
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let ctx = Ctx::from_parts(&app_state, parts);
        Ok(ctx)
    }
}
