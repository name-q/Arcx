//! 定时任务调度管理器
//!
//! 负责：
//! - 注册所有 ScheduleJob
//! - 在应用启动时开始调度
//! - 在应用关闭时停止调度

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use tokio_cron_scheduler::{Job, JobScheduler};

use super::{JobContext, ScheduleJob};
use crate::config::AppConfig;

/// 调度管理器
pub struct ScheduleManager {
    jobs: Vec<Box<dyn ScheduleJob>>,
    scheduler: Option<JobScheduler>,
}

impl ScheduleManager {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            scheduler: None,
        }
    }

    /// 注册一个定时任务
    pub fn register(&mut self, job: impl ScheduleJob) {
        tracing::info!("  Schedule job registered: {} [{}]", job.name(), job.cron());
        self.jobs.push(Box::new(job));
    }

    /// 注册一个已装箱的定时任务
    pub fn register_boxed(&mut self, job: Box<dyn ScheduleJob>) {
        tracing::info!("  Schedule job registered: {} [{}]", job.name(), job.cron());
        self.jobs.push(job);
    }

    /// 启动调度器
    /// 将所有注册的任务添加到 cron 调度器并开始执行
    pub async fn start(
        &mut self,
        config: Arc<AppConfig>,
        resources: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    ) -> Result<(), String> {
        if self.jobs.is_empty() {
            tracing::info!("No schedule jobs registered, skipping scheduler start");
            return Ok(());
        }

        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| format!("Failed to create scheduler: {}", e))?;

        // 为每个任务创建 cron job
        for job_def in self.jobs.drain(..) {
            let job_name = job_def.name().to_string();
            let cron_expr = job_def.cron().to_string();

            let ctx = Arc::new(JobContext::new(config.clone(), resources.clone()));
            let job_arc: Arc<dyn ScheduleJob> = Arc::from(job_def);

            let cron_job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
                let ctx = ctx.clone();
                let job = job_arc.clone();
                let name = job_name.clone();
                Box::pin(async move {
                    tracing::info!("[Schedule] Running: {}", name);
                    job.run(&ctx).await;
                    tracing::info!("[Schedule] Completed: {}", name);
                })
            })
            .map_err(|e| format!("Failed to create job: {}", e))?;

            scheduler
                .add(cron_job)
                .await
                .map_err(|e| format!("Failed to add job: {}", e))?;
        }

        // 启动调度器
        scheduler
            .start()
            .await
            .map_err(|e| format!("Failed to start scheduler: {}", e))?;

        tracing::info!("Schedule system started");
        self.scheduler = Some(scheduler);
        Ok(())
    }

    /// 停止调度器
    pub async fn shutdown(&mut self) {
        if let Some(mut scheduler) = self.scheduler.take() {
            if let Err(e) = scheduler.shutdown().await {
                tracing::error!("Schedule shutdown error: {}", e);
            } else {
                tracing::info!("Schedule system shut down");
            }
        }
    }
}
