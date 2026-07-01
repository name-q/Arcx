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
//! mod controller;
//! mod router;
//!
//! #[tokio::main]
//! async fn main() {
//!     Arcx::new()
//!         .routes(router::routes)
//!         .run()
//!         .await;
//! }
//! ```

#![allow(dead_code)]

pub mod client;
pub mod config;
pub mod context;
pub mod ctx;
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
    pub use crate::config::{AppConfig, FromTomlValue};
    pub use crate::context::AppState;
    pub use crate::ctx::{Ctx, Service};
    pub use crate::error::{AppError, AppResult, FieldError};
    pub use crate::extract::ValidJson;
    pub use crate::guard::{auth_guard, AuthProvider, AuthUser};
    pub use crate::guard::auth::RequestParts;
    pub use crate::httpclient::HttpClient;
    pub use crate::lifecycle::{Lifecycle, ShutdownSignal, ShutdownTrigger, shutdown_channel};
    pub use crate::plugin::{Plugin, PluginError, PluginManager};
    pub use crate::router::{ArcxRouter, ResourceHandlers};
    pub use crate::schedule::{ScheduleJob, JobContext};
    pub use crate::session::Session;
    pub use crate::ws::{WsHandler, WsMessage, WsSession, WsRegistry};
    pub use crate::client::event_bus::{EventBus, AppEvent};
    pub use crate::client::{Client, Subscriber, Invoker, ClientError};
    pub use crate::config::watcher::ConfigWatcher;
    pub use crate::middleware::apply_global_middleware;

    // 兼容旧代码：Context 作为 Ctx 的别名
    #[deprecated(since = "0.1.4", note = "Use `Ctx` instead of `Context`")]
    pub type Context = crate::ctx::Ctx;

    // Re-export 常用第三方依赖
    pub use axum;
    pub use axum::extract::Path;
    pub use axum::http::StatusCode;
    pub use axum::response::IntoResponse;
    pub use axum::Json;
    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{self, json, Value};
    pub use validator::Validate;
    pub use tracing;
    pub use tokio;
}

use std::any::TypeId;
use std::sync::Arc;

use guard::AuthProvider;

/// Arcx 框架入口
///
/// Builder 模式启动框架，内部自动完成：
/// - 配置加载
/// - 日志初始化
/// - 插件加载
/// - EventBus & ConfigWatcher
/// - AppState 构建
/// - 路由组装 & 中间件
/// - HTTP 服务启动 & graceful shutdown
///
/// ## 用法
///
/// ```rust,no_run
/// Arcx::new()
///     .auth(my_auth_provider)
///     .routes(router::routes)
///     .run()
///     .await;
/// ```
pub struct Arcx {
    routes_fn: Option<Box<dyn FnOnce(&mut router::ArcxRouter) + Send>>,
    auth_provider: Option<Box<dyn AuthProvider>>,
}

impl Arcx {
    /// 创建 Arcx 实例
    pub fn new() -> Self {
        Self {
            routes_fn: None,
            auth_provider: None,
        }
    }

    /// 注册鉴权提供者
    ///
    /// 开启后 `guarded_scope` 内的路由自动调用 provider 验证。
    /// 不注册则不能使用 `guarded_scope`（运行时会报错）。
    pub fn auth(mut self, provider: impl AuthProvider) -> Self {
        self.auth_provider = Some(Box::new(provider));
        self
    }

    /// 注册路由（传入 router.rs 中的 routes 函数）
    pub fn routes(mut self, f: fn(&mut router::ArcxRouter)) -> Self {
        self.routes_fn = Some(Box::new(f));
        self
    }

    /// 启动服务
    ///
    /// 内部完成所有初始化流程并监听 HTTP 端口。
    /// 支持 Ctrl+C graceful shutdown。
    pub async fn run(self) {
        // 1. 加载配置
        let cfg = config::AppConfig::load();

        // 2. 初始化日志
        logger::init(&cfg.logger);
        tracing::info!("{} v{} starting...", cfg.app.name, cfg.app.version);
        tracing::info!("Environment: {}", cfg.app.env);

        // 3. 初始化插件
        let raw_config = config::load_raw_config();
        let mut plugin_manager = plugin::PluginManager::new();
        plugin_manager.auto_register_builtin(&raw_config);
        if let Err(e) = plugin_manager.init_all(&raw_config).await {
            tracing::error!("Plugin init failed: {}", e);
            std::process::exit(1);
        }

        // 4. EventBus & ConfigWatcher
        let event_bus = client::event_bus::EventBus::new(128);
        let (_notifier, config_watcher) = config::watcher::ConfigWatcher::new(cfg.clone());

        // 5. HttpClient
        let http_client = httpclient::HttpClient::new(cfg.httpclient.clone());

        // 6. 构建 AppState
        let mut resources = plugin_manager.take_resources();
        resources.insert(
            TypeId::of::<httpclient::HttpClient>(),
            Arc::new(http_client),
        );
        let mut state = context::AppState::with_resources(
            cfg.clone(),
            resources,
            event_bus.clone(),
            config_watcher,
        );

        // 7. 注册 AuthProvider（如果有）
        if let Some(provider) = self.auth_provider {
            state.set_auth_provider_boxed(provider);
            tracing::info!("Auth provider registered");
        }

        // 8. 构建路由
        let mut arcx_router = router::ArcxRouter::new();
        if let Some(routes_fn) = self.routes_fn {
            tracing::info!("Loading routes...");
            routes_fn(&mut arcx_router);
        }
        let app_router = arcx_router.build(&state);

        // 9. 应用中间件
        let app = middleware::apply_global_middleware(app_router, &cfg).with_state(state);

        // 10. 启动服务
        let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
        tracing::info!("Arcx server running at http://{}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.ok();
                tracing::info!("Shutting down gracefully...");
            })
            .await
            .unwrap();

        // 11. 清理
        plugin_manager.shutdown_all().await;
        tracing::info!("Server stopped.");
    }

    // ─── 兼容旧用法的静态方法 ───────────────────────────

    /// 加载强类型配置（用于需要手动控制启动流程的场景）
    pub fn load_config() -> config::AppConfig {
        config::AppConfig::load()
    }

    /// 加载原始配置（用于插件系统）
    pub fn load_raw_config() -> toml::Value {
        config::load_raw_config()
    }
}

impl Default for Arcx {
    fn default() -> Self {
        Self::new()
    }
}
