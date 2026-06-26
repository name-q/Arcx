//! 内置 HTTP 客户端
//!
//! 封装 reqwest，提供框架级的 HTTP 调用能力：
//! - 配置化（超时、重试、base_url）
//! - 请求/响应钩子（可用于日志、监控）
//! - 自动 JSON 序列化/反序列化
//! - 统一错误类型
//!
//! 配置方式：
//! ```toml
//! [httpclient]
//! timeout = 30          # 超时秒数
//! max_retries = 0       # 重试次数
//! user_agent = "Arcx/0.1"
//! ```
//!
//! 用法（在 Service/Controller 中）：
//! ```rust
//! let client = ctx.resource::<HttpClient>().unwrap();
//! let resp = client.get("https://api.example.com/data").send().await?;
//! let body: MyData = resp.json().await?;
//! ```

use reqwest::{Client, ClientBuilder, Method, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;

/// HTTP 客户端配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HttpClientConfig {
    /// 请求超时（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// 最大重试次数
    #[serde(default)]
    pub max_retries: u32,
    /// User-Agent
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    /// 连接超时（秒）
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            max_retries: 0,
            user_agent: default_user_agent(),
            connect_timeout: default_connect_timeout(),
        }
    }
}

fn default_timeout() -> u64 { 30 }
fn default_connect_timeout() -> u64 { 10 }
fn default_user_agent() -> String { "Arcx/0.1".to_string() }

/// HTTP 客户端错误
#[derive(Debug)]
pub enum HttpError {
    /// 请求失败
    Request(reqwest::Error),
    /// 重试耗尽
    RetryExhausted { attempts: u32, last_error: reqwest::Error },
    /// JSON 解析失败
    Deserialize(reqwest::Error),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) => write!(f, "HTTP request failed: {}", e),
            Self::RetryExhausted { attempts, last_error } => {
                write!(f, "HTTP request failed after {} attempts: {}", attempts, last_error)
            }
            Self::Deserialize(e) => write!(f, "Response deserialize failed: {}", e),
        }
    }
}

impl std::error::Error for HttpError {}

/// 框架内置 HTTP 客户端
/// 基于 reqwest，增加重试、日志、配置化
#[derive(Clone)]
pub struct HttpClient {
    inner: Client,
    config: HttpClientConfig,
}

impl HttpClient {
    /// 从配置创建
    pub fn new(config: HttpClientConfig) -> Self {
        let inner = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout))
            .connect_timeout(Duration::from_secs(config.connect_timeout))
            .user_agent(&config.user_agent)
            .build()
            .expect("Failed to build HTTP client");

        Self { inner, config }
    }

    /// 从默认配置创建
    pub fn default_client() -> Self {
        Self::new(HttpClientConfig::default())
    }

    /// GET 请求
    pub fn get(&self, url: &str) -> RequestBuilder {
        self.inner.get(url)
    }

    /// POST 请求
    pub fn post(&self, url: &str) -> RequestBuilder {
        self.inner.post(url)
    }

    /// PUT 请求
    pub fn put(&self, url: &str) -> RequestBuilder {
        self.inner.put(url)
    }

    /// DELETE 请求
    pub fn delete(&self, url: &str) -> RequestBuilder {
        self.inner.delete(url)
    }

    /// PATCH 请求
    pub fn patch(&self, url: &str) -> RequestBuilder {
        self.inner.patch(url)
    }

    /// 通用请求方法
    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        self.inner.request(method, url)
    }

    /// 带重试的 GET + JSON 解析（便捷方法）
    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, HttpError> {
        let resp = self.send_with_retry(Method::GET, url, None::<&()>).await?;
        resp.json::<T>().await.map_err(HttpError::Deserialize)
    }

    /// 带重试的 POST + JSON 解析（便捷方法）
    pub async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T, HttpError> {
        let resp = self.send_with_retry(Method::POST, url, Some(body)).await?;
        resp.json::<T>().await.map_err(HttpError::Deserialize)
    }

    /// 带重试机制的请求发送
    async fn send_with_retry<B: Serialize>(
        &self,
        method: Method,
        url: &str,
        body: Option<&B>,
    ) -> Result<Response, HttpError> {
        let max_attempts = self.config.max_retries + 1;
        let mut last_error = None;

        for attempt in 1..=max_attempts {
            let mut req = self.inner.request(method.clone(), url);
            if let Some(b) = body {
                req = req.json(b);
            }

            match req.send().await {
                Ok(resp) => {
                    tracing::debug!(
                        "HTTP {} {} -> {} (attempt {})",
                        method, url, resp.status(), attempt
                    );
                    return Ok(resp);
                }
                Err(e) => {
                    tracing::warn!(
                        "HTTP {} {} failed (attempt {}/{}): {}",
                        method, url, attempt, max_attempts, e
                    );
                    last_error = Some(e);

                    // 重试前等待（指数退避）
                    if attempt < max_attempts {
                        let delay = Duration::from_millis(100 * 2u64.pow(attempt - 1));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(HttpError::RetryExhausted {
            attempts: max_attempts,
            last_error: last_error.unwrap(),
        })
    }

    /// 获取底层 reqwest Client（高级用法）
    pub fn raw(&self) -> &Client {
        &self.inner
    }
}
