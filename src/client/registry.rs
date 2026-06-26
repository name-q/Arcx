//! 客户端注册表
//!
//! 负责：
//! - 管理所有 Client 的生命周期（连接 → 健康检查 → 断开）
//! - 后台心跳任务（tokio::spawn，不是 Arc 轮询）
//! - 客户端异常时通过 EventBus 广播事件
//!
//! 设计：
//! - 注册时 connect
//! - 启动后台心跳（间隔可配）
//! - 心跳失败时发 HealthCheckFailed 事件
//! - shutdown 时逆序 disconnect

use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinHandle;

use super::{Client, ClientError};
use super::event_bus::{AppEvent, EventBus};

/// 注册表中的客户端条目
struct ClientEntry {
    client: Arc<dyn Client>,
    health_interval: Duration,
}

/// 客户端注册表
pub struct ClientRegistry {
    clients: Vec<ClientEntry>,
    event_bus: EventBus,
    health_tasks: Vec<JoinHandle<()>>,
}

impl ClientRegistry {
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            clients: Vec::new(),
            event_bus,
            health_tasks: Vec::new(),
        }
    }

    /// 注册并连接客户端
    /// health_interval: 健康检查间隔（None = 不做心跳）
    pub async fn register(
        &mut self,
        client: Arc<dyn Client>,
        health_interval: Option<Duration>,
    ) -> Result<(), ClientError> {
        let name = client.name().to_string();
        tracing::info!("  Client connecting: {}", name);

        // 建立连接
        client.connect().await?;

        tracing::info!("  Client connected: {} ✓", name);
        self.event_bus.emit(AppEvent::ClientConnected { name: name.clone() });

        let interval = health_interval.unwrap_or(Duration::from_secs(30));
        self.clients.push(ClientEntry {
            client,
            health_interval: interval,
        });

        Ok(())
    }

    /// 启动所有客户端的后台健康检查
    /// 每个客户端一个独立的 tokio task（轻量，不占线程）
    pub fn start_health_checks(&mut self) {
        for entry in &self.clients {
            let client = entry.client.clone();
            let interval = entry.health_interval;
            let event_bus = self.event_bus.clone();
            let name = client.name().to_string();

            let handle = tokio::spawn(async move {
                let mut ticker = tokio::time::interval(interval);
                ticker.tick().await; // 跳过第一次（刚连上不需要立刻检查）

                loop {
                    ticker.tick().await;

                    if !client.is_connected() {
                        event_bus.emit(AppEvent::ClientDisconnected {
                            name: name.clone(),
                        });
                        break;
                    }

                    match client.health_check().await {
                        Ok(true) => {
                            tracing::trace!("Client {} health check OK", name);
                        }
                        Ok(false) => {
                            tracing::warn!("Client {} health check: unhealthy", name);
                            event_bus.emit(AppEvent::HealthCheckFailed {
                                name: name.clone(),
                                reason: "health_check returned false".to_string(),
                            });
                        }
                        Err(e) => {
                            tracing::error!("Client {} health check error: {}", name, e);
                            event_bus.emit(AppEvent::HealthCheckFailed {
                                name: name.clone(),
                                reason: e.message,
                            });
                        }
                    }
                }
            });

            self.health_tasks.push(handle);
        }

        if !self.clients.is_empty() {
            tracing::info!(
                "Health checks started for {} client(s)",
                self.clients.len()
            );
        }
    }

    /// 优雅关闭所有客户端（逆序）
    pub async fn shutdown(&mut self) {
        // 停止所有心跳任务
        for handle in self.health_tasks.drain(..) {
            handle.abort();
        }

        // 逆序断开连接
        for entry in self.clients.iter().rev() {
            let name = entry.client.name().to_string();
            tracing::info!("  Client disconnecting: {}", name);
            if let Err(e) = entry.client.disconnect().await {
                tracing::error!("  Client disconnect error: {}", e);
            } else {
                self.event_bus.emit(AppEvent::ClientDisconnected { name: name.clone() });
                tracing::info!("  Client disconnected: {} ✓", name);
            }
        }

        self.clients.clear();
    }

    /// 获取已注册客户端数量
    pub fn count(&self) -> usize {
        self.clients.len()
    }
}
