//! 路由守卫系统
//!
//! Guard 是路由级别的中间件，用于请求级权限控制。
//! 框架定义 `AuthProvider` trait，用户实现自己的验证逻辑。
//!
//! ## 设计理念
//!
//! - 框架不绑定任何具体鉴权方案（JWT/Session/OAuth...）
//! - 用户实现 `AuthProvider::authenticate`，控制 token 从哪取、怎么验
//! - `guarded_scope` 内的路由自动调用用户注册的 provider
//! - 验证通过后 `AuthUser` 注入请求，handler 直接提取
//!
//! ## 用法
//!
//! ```rust
//! // 1. 实现 AuthProvider（用户代码）
//! pub struct MyAuth { /* ... */ }
//!
//! #[async_trait]
//! impl AuthProvider for MyAuth {
//!     async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError> {
//!         // 从 header/cookie 取 token，验证，返回 AuthUser
//!     }
//! }
//!
//! // 2. 注册到 Arcx
//! Arcx::new()
//!     .auth(MyAuth::new())
//!     .routes(router::routes)
//!     .run().await;
//!
//! // 3. 路由中使用
//! r.guarded_scope("/api/admin", |s| {
//!     s.get("/profile", controller::admin::profile);
//! });
//!
//! // 4. handler 中提取
//! pub async fn profile(ctx: Context, user: AuthUser) -> AppResult<impl IntoResponse> {
//!     Ok(Json(json!({ "id": user.id })))
//! }
//! ```

pub mod auth;

pub use auth::{auth_guard, AuthProvider, AuthUser};
