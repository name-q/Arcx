//! Schedule 定时任务系统
//!
//! 约定：
//! - 实现 ScheduleJob trait 定义任务
//! - 每个任务指定 cron 表达式和执行逻辑
//! - 框架启动时自动注册并开始调度
//! - 配置 [schedule] enable = true 启用
//!
//! 用法：
//! ```rust
//! pub struct CleanExpiredTokens;
//!
//! #[async_trait]
//! impl ScheduleJob for CleanExpiredTokens {
//!     fn name(&self) -> &str { "clean_expired_tokens" }
//!     fn cron(&self) -> &str { "0 0 * * * *" } // 每小时
//!     async fn run(&self, ctx: &JobContext) {
//!         // 清理逻辑
//!     }
//! }
//! ```

pub mod manager;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::AppConfig;

/// 任务执行上下文
/// 提供对共享资源的访问（config、插件资源）
pub struct JobContext {
    pub config: Arc<AppConfig>,
    resources: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl JobContext {
    pub fn new(
        config: Arc<AppConfig>,
        resources: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    ) -> Self {
        Self { config, resources }
    }

    /// 获取插件资源（和 Context 同样的方式）
    pub fn resource<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.clone().downcast::<T>().ok())
    }
}

/// 定时任务 trait
/// 每个定时任务必须实现此 trait
#[async_trait::async_trait]
pub trait ScheduleJob: Send + Sync + 'static {
    /// 任务名称（用于日志标识）
    fn name(&self) -> &str;

    /// Cron 表达式（6位：秒 分 时 日 月 周）
    /// 示例：
    /// - "0 */5 * * * *"  每5分钟
    /// - "0 0 * * * *"    每小时
    /// - "0 0 2 * * *"    每天凌晨2点
    /// - "0 0 0 * * 1"    每周一凌晨
    fn cron(&self) -> &str;

    /// 执行任务逻辑
    async fn run(&self, ctx: &JobContext);
}
