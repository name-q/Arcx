//! 配置热更新 —— 基于 tokio::sync::watch
//!
//! 设计：
//! - 配置变更时，通过 watch channel 推送新值
//! - 订阅者拿到 Receiver，自动获取最新配置
//! - watch channel 特性：只保留最新值，新订阅者立刻拿到当前值
//!
//! 为什么用 watch 而不是 Arc<RwLock<Config>>：
//! - watch 是单生产者多消费者的值通知模型
//! - 消费者通过 changed() 等待通知，不需要轮询
//! - 不需要加锁读取，性能更好
//! - 语义更清晰：「配置变了」 vs 「拿锁读共享状态」
//!
//! 用法：
//! ```rust
//! let (notifier, watcher) = ConfigWatcher::new(initial_config);
//!
//! // 后台监听变更
//! let mut rx = watcher.subscribe();
//! tokio::spawn(async move {
//!     while rx.changed().await.is_ok() {
//!         let new_config = rx.borrow().clone();
//!         println!("Config updated: {:?}", new_config);
//!     }
//! });
//!
//! // 触发更新
//! notifier.update(new_config);
//! ```

use tokio::sync::watch;

use super::AppConfig;

/// 配置变更通知者（写端）
/// 通常由配置加载模块持有
pub struct ConfigNotifier {
    tx: watch::Sender<AppConfig>,
}

impl ConfigNotifier {
    /// 推送新配置
    /// 所有通过 ConfigWatcher::subscribe() 获取的 Receiver 都会收到通知
    pub fn update(&self, config: AppConfig) {
        // send 不需要 await，同步操作
        let _ = self.tx.send(config);
    }

    /// 获取当前配置的引用
    pub fn current(&self) -> watch::Ref<'_, AppConfig> {
        self.tx.borrow()
    }
}

/// 配置观察者（读端工厂）
/// 注入到 AppState，任何需要监听配置变更的地方调用 subscribe()
#[derive(Clone)]
pub struct ConfigWatcher {
    rx: watch::Receiver<AppConfig>,
}

impl ConfigWatcher {
    /// 创建配置热更新通道
    /// 返回 (通知者, 观察者)
    pub fn new(initial: AppConfig) -> (ConfigNotifier, Self) {
        let (tx, rx) = watch::channel(initial);
        (ConfigNotifier { tx }, Self { rx })
    }

    /// 获取一个新的订阅 Receiver
    /// 调用者可以 .changed().await 等待配置变更
    pub fn subscribe(&self) -> watch::Receiver<AppConfig> {
        self.rx.clone()
    }

    /// 获取当前配置（不等待变更，立刻返回）
    pub fn current(&self) -> AppConfig {
        self.rx.borrow().clone()
    }
}
