//! 示例定时任务
//! 
//! 演示 Schedule 系统用法。实际项目中可按业务分文件放在 service/ 或 schedule/ 目录。

use crate::schedule::{JobContext, ScheduleJob};

/// 健康检查定时任务（每30秒执行）
/// 演示如何访问 config 和插件资源
pub struct HealthCheckJob;

#[async_trait::async_trait]
impl ScheduleJob for HealthCheckJob {
    fn name(&self) -> &str {
        "health_check"
    }

    fn cron(&self) -> &str {
        // 每30秒执行一次（仅演示用，实际可设为每分钟或每小时）
        "*/30 * * * * *"
    }

    async fn run(&self, ctx: &JobContext) {
        let app_name = &ctx.config.app.name;
        tracing::info!("[HealthCheck] {} is running, env={}", app_name, ctx.config.app.env);
    }
}
