//! Service 层约定
//! 
//! Service 是业务逻辑的承载层，位于 Controller 和 Model 之间。
//! 
//! 框架约定：
//! - Service 定义为 struct，方法通过 impl 实现
//! - Service 可以持有对资源的引用（db 连接、外部 client 等）
//! - Controller 通过 Context 获取资源后构造或调用 Service
//! 
//! 示例写法：
//! ```rust
//! pub struct UserService;
//! 
//! impl UserService {
//!     pub async fn find_by_id(db: &DbPool, id: i64) -> AppResult<User> {
//!         // ...
//!     }
//! }
//! ```
//! 
//! 设计说明：
//! - Service 本身是无状态的（不持有 self 可变引用）
//! - 需要的依赖通过参数传入，而非全局单例
//! - 这使得 Service 天然支持并发和测试

pub mod demo_jobs;
