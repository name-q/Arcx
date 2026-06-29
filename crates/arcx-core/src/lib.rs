//! # Arcx Framework
//!
//! A convention-over-configuration web framework for Rust,
//! built on top of Axum with AI orchestration capabilities.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use arcx_core::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let cfg = Arcx::load_config();
//!     arcx_core::logger::init(&cfg.logger);
//!     // ... build app
//! }
//! ```

#![allow(dead_code)]

pub mod client;
pub mod config;
pub mod context;
pub mod error;
pub mod extract;
pub mod guard;
pub mod httpclient;
pub mod lifecycle;
pub mod logger;
pub mod middleware;
pub mod plugin;
pub mod router;
pub mod schedule;
pub mod session;
pub mod ws;

// Re-export 核心类型
pub mod prelude {
    pub use crate::Arcx;
    pub use crate::config::AppConfig;
    pub use crate::context::{AppState, Context};
    pub use crate::error::{AppError, AppResult, success};
    pub use crate::extract::ValidJson;
    pub use crate::guard::{auth_guard, CurrentUser};
    pub use crate::httpclient::HttpClient;
    pub use crate::lifecycle::{Lifecycle, ShutdownSignal, ShutdownTrigger, shutdown_channel};
    pub use crate::plugin::{Plugin, PluginError, PluginManager};
    pub use crate::schedule::{ScheduleJob, JobContext};
    pub use crate::session::Session;
    pub use crate::ws::{WsHandler, WsMessage, WsSession, WsRegistry};
    pub use crate::client::event_bus::{EventBus, AppEvent};
    pub use crate::client::{Client, Subscriber, Invoker, ClientError};
    pub use crate::config::watcher::ConfigWatcher;
    pub use crate::middleware::apply_global_middleware;

    // Re-export 常用第三方依赖
    pub use axum;
    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{self, json};
    pub use validator::Validate;
    pub use tracing;
    pub use tokio;
}

/// Arcx 框架入口
/// 提供配置加载等静态方法
pub struct Arcx;

impl Arcx {
    /// 加载强类型配置
    pub fn load_config() -> config::AppConfig {
        config::AppConfig::load()
    }

    /// 加载原始配置（用于插件系统）
    pub fn load_raw_config() -> toml::Value {
        config::load_raw_config()
    }
}
