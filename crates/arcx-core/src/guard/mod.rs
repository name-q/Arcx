//! 路由守卫系统
//! 
//! Guard 是路由级别的中间件，用于请求级权限控制。
//! 不同于全局中间件（每个请求都经过），Guard 只作用于特定路由。
//! 
//! 框架内置守卫：
//! - AuthGuard: 要求请求携带有效 JWT token
//! 
//! 用法（在 controller 路由中使用）：
//! ```rust
//! pub fn routes() -> Router<AppState> {
//!     Router::new()
//!         .route("/profile", get(profile))
//!         .layer(axum::middleware::from_fn_with_state(state, auth_guard))
//! }
//! ```
//! 
//! 或使用路由分组：
//! ```rust
//! pub fn routes() -> Router<AppState> {
//!     let public = Router::new()
//!         .route("/login", post(login));
//!     let protected = Router::new()
//!         .route("/profile", get(profile))
//!         .route_layer(axum::middleware::from_fn(auth_guard));
//!     
//!     public.merge(protected)
//! }
//! ```

pub mod auth;

pub use auth::auth_guard;
pub use auth::CurrentUser;
