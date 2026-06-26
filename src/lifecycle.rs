//! 应用生命周期管理
//!
//! 设计升级：
//! - 之前：Arc<RwLock<PluginManager>> 传来传去
//! - 现在：mpsc channel 驱动，生命周期事件作为消息发送
//!
//! 为什么用 channel：
//! - 生命周期是「事件流」，不是「共享状态」
//! - shutdown 信号只发一次，用 oneshot 最合适
//! - 启动阶段各步骤有明确顺序，用 Vec<Hook> + 依次执行即可
//! - 不再需要把 PluginManager 的引用传满整个 app

use tokio::sync::oneshot;

use crate::client::event_bus::{AppEvent, EventBus};

/// 生命周期阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// 初始化中（加载配置、注册插件）
    Initializing,
    /// 启动中（连接数据库、启动服务）
    Starting,
    /// 运行中
    Running,
    /// 关闭中
    Stopping,
    /// 已停止
    Stopped,
}

/// 关闭信号发送者
/// main 函数持有，收到 ctrl+c 时触发
pub struct ShutdownTrigger {
    tx: oneshot::Sender<()>,
}

impl ShutdownTrigger {
    /// 触发关闭
    pub fn shutdown(self) {
        let _ = self.tx.send(());
    }
}

/// 关闭信号接收者
/// 各子系统持有，await 即可等待关闭信号
#[derive(Clone)]
pub struct ShutdownSignal {
    rx: ShutdownReceiver,
}

/// 内部用 broadcast 实现多消费者（oneshot 只能单消费者）
#[derive(Clone)]
struct ShutdownReceiver {
    inner: tokio::sync::watch::Receiver<bool>,
}

/// 创建关闭信号对
/// 返回 (trigger, signal)
/// - trigger: 只有一个，调用 shutdown() 通知所有人
/// - signal: 可以 clone，分发给所有子系统
pub fn shutdown_channel() -> (ShutdownTrigger, ShutdownSignal) {
    let (watch_tx, watch_rx) = tokio::sync::watch::channel(false);

    // 用 oneshot 触发 → watch 广播给所有人
    let (tx, rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let _ = rx.await;
        let _ = watch_tx.send(true);
    });

    let signal = ShutdownSignal {
        rx: ShutdownReceiver { inner: watch_rx },
    };

    (ShutdownTrigger { tx }, signal)
}

impl ShutdownSignal {
    /// 等待关闭信号
    pub async fn wait(&mut self) {
        // 等到值变为 true
        while !*self.rx.inner.borrow() {
            if self.rx.inner.changed().await.is_err() {
                break;
            }
        }
    }
}

/// 关闭回调（async 闭包）
type ShutdownHook = Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send>;

/// 生命周期管理器
pub struct Lifecycle {
    phase: Phase,
    event_bus: EventBus,
    shutdown_hooks: Vec<ShutdownHook>,
}

impl Lifecycle {
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            phase: Phase::Initializing,
            event_bus,
            shutdown_hooks: Vec::new(),
        }
    }

    /// 获取当前阶段
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// 注册关闭钩子（LIFO 执行）
    pub fn on_shutdown<F, Fut>(&mut self, hook: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.shutdown_hooks.push(Box::new(move || Box::pin(hook())));
    }

    /// 标记启动完成
    pub fn mark_started(&mut self) {
        self.phase = Phase::Running;
        self.event_bus.emit(AppEvent::Started);
    }

    /// 执行完整的关闭流程
    pub async fn shutdown(mut self) {
        self.phase = Phase::Stopping;
        self.event_bus.emit(AppEvent::Stopping);
        tracing::info!("Graceful shutdown starting...");

        // 逆序执行关闭钩子
        let hooks: Vec<_> = self.shutdown_hooks.drain(..).rev().collect();
        for hook in hooks {
            hook().await;
        }

        self.phase = Phase::Stopped;
        tracing::info!("Graceful shutdown complete");
    }
}
