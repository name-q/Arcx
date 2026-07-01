//! Ctx — 请求作用域的服务定位器
//!
//! 核心能力：
//! - 配置读取：ctx.get("redis.url")、ctx.env()
//! - 插件获取：ctx.plugin::<T>()
//! - 服务定位：ctx.service::<T>()（同请求内懒创建+缓存）
//! - 状态读写：ctx.set(val)、ctx.state::<T>()（请求级数据共享）
//!
//! ## 设计哲学
//!
//! Ctx 是可选的。不需要时不写在 handler 参数里：
//! ```rust
//! // 不需要 Ctx
//! pub async fn destroy(Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
//!     Ok(response::no_content())
//! }
//!
//! // 需要 Ctx
//! pub async fn show(ctx: Ctx, Path(id): Path<u64>) -> AppResult<impl IntoResponse> {
//!     let user = ctx.service::<UserService>().find(id).await?;
//!     Ok(Json(user))
//! }
//! ```
//!
//! ## Service 互调
//!
//! Service 持有 Ctx（Arc 内部，clone 零成本），通过 ctx.service::<T>() 互调：
//! ```rust
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
use std::sync::{Arc, RwLock};

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;

use crate::config::{AppConfig, FromTomlValue};
use crate::context::AppState;
use crate::error::{AppError, AppResult};
use crate::plugin::Plugin;

/// Service trait — 用户 Service 必须实现
///
/// 框架通过此 trait 懒创建 Service 实例并缓存。
///
/// ## 示例
///
/// ```rust
/// pub struct UserService {
///     ctx: Ctx,
/// }
///
/// impl Service for UserService {
///     fn create(ctx: &Ctx) -> Arc<Self> {
///         Arc::new(Self { ctx: ctx.clone() })
///     }
/// }
///
/// impl UserService {
///     pub async fn find_by_id(&self, id: i64) -> AppResult<User> {
///         // 业务逻辑...
///     }
///
///     pub async fn find_with_orders(&self, id: i64) -> AppResult<UserWithOrders> {
///         let user = self.find_by_id(id).await?;
///         // Service 互调
///         let orders = self.ctx.service::<OrderService>().find_by_user(id).await?;
///         Ok(UserWithOrders { user, orders })
///     }
/// }
/// ```
pub trait Service: Send + Sync + 'static {
    /// 创建 Service 实例
    ///
    /// 每个请求内首次调用时执行，后续复用缓存。
    /// ctx 参数用于获取配置、插件、或互调其他 Service。
    fn create(ctx: &Ctx) -> Arc<Self>
    where
        Self: Sized;
}

/// 请求作用域的服务定位器 + 状态容器
///
/// - 可选：不需要就不写在 handler 参数里
/// - clone 零成本：内部全是 Arc
/// - 请求结束自动释放所有 Service 实例和状态
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
    /// Service 实例缓存（同一请求内复用）
    services: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    /// 请求级状态（中间件/Service 可读写）
    state: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl Ctx {
    /// 从 AppState 创建（框架内部使用）
    pub(crate) fn from_app_state(app_state: &AppState) -> Self {
        Self {
            app: Arc::new(AppInner {
                config: app_state.config.clone(),
                resources: app_state.resources_ref().clone(),
            }),
            request: Arc::new(RequestInner {
                services: RwLock::new(HashMap::new()),
                state: RwLock::new(HashMap::new()),
            }),
        }
    }

    // ─── 配置 ─────────────────────────────────

    /// 获取当前环境名
    ///
    /// ```rust
    /// let env = ctx.env(); // "dev" | "prod" | "test"
    /// ```
    pub fn env(&self) -> &str {
        &self.app.config.app.env
    }

    /// 是否为开发环境
    pub fn is_dev(&self) -> bool {
        self.app.config.app.env == "dev"
    }

    /// 通过 dot-notation 路径获取配置值
    ///
    /// ```rust
    /// let url: Option<String> = ctx.get("redis.url");
    /// let pool: Option<u64> = ctx.get("redis.pool_size");
    /// let port: Option<u16> = ctx.get("server.port");
    /// ```
    pub fn get<T: FromTomlValue>(&self, path: &str) -> Option<T> {
        self.app.config.get(path)
    }

    /// 将配置段反序列化为自定义结构
    ///
    /// ```rust
    /// #[derive(Deserialize)]
    /// struct RedisConfig { url: String, pool_size: u32 }
    /// let redis: Option<RedisConfig> = ctx.get_as("redis");
    /// ```
    pub fn get_as<T: serde::de::DeserializeOwned>(&self, path: &str) -> Option<T> {
        self.app.config.get_as(path)
    }

    /// 获取强类型配置引用
    pub fn config(&self) -> &AppConfig {
        &self.app.config
    }

    // ─── 插件 ─────────────────────────────────

    /// 获取插件实例
    ///
    /// ```rust
    /// let db = ctx.plugin::<DatabasePlugin>()?;
    /// let jwt = ctx.plugin::<JwtPlugin>()?;
    /// ```
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
    ///
    /// ```rust
    /// let conn = ctx.resource::<DatabaseConnection>();
    /// ```
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
    /// Service 间可通过此方法互相调用。
    ///
    /// ```rust
    /// let user_svc = ctx.service::<UserService>();
    /// let result = user_svc.find_by_id(1).await?;
    /// ```
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

    // ─── 请求级状态 ─────────────────────────────────

    /// 写入请求级状态
    ///
    /// 中间件或 Service 可写入数据，后续层通过 state::<T>() 读取。
    ///
    /// ```rust
    /// ctx.set(AuthUser { id: "123".into(), payload: json!({}) });
    /// ```
    pub fn set<T: Send + Sync + 'static>(&self, val: T) {
        let mut state = self.request.state.write().unwrap();
        state.insert(TypeId::of::<T>(), Arc::new(val));
    }

    /// 读取请求级状态
    ///
    /// ```rust
    /// let user = ctx.state::<AuthUser>();
    /// ```
    pub fn state<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let state = self.request.state.read().unwrap();
        state
            .get(&TypeId::of::<T>())
            .and_then(|v| v.clone().downcast::<T>().ok())
    }
}

/// 实现 FromRequestParts，让 Ctx 能作为 handler 参数自动提取
#[axum::async_trait]
impl<S> FromRequestParts<S> for Ctx
where
    AppState: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        Ok(Ctx::from_app_state(&app_state))
    }
}
