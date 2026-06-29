//! Cookie & Session
//!
//! 提供 Web 状态管理能力：
//! - Cookie 读写（签名/加密可选）
//! - Session 抽象（默认 Cookie 存储，可扩展 Redis 等后端）
//! - 配置化
//!
//! 配置方式：
//! ```toml
//! [session]
//! secret = "your-secret-key-at-least-32-chars!!"
//! cookie_name = "arcx.sid"
//! max_age = 86400          # 秒
//! http_only = true
//! secure = false           # 生产环境设 true
//! same_site = "lax"        # strict / lax / none
//! ```
//!
//! 用法：
//! ```rust
//! async fn handler(session: Session) -> impl IntoResponse {
//!     // 读取
//!     let user_id = session.get::<String>("user_id");
//!     // 写入
//!     session.set("user_id", "123");
//!     // 删除
//!     session.remove("user_id");
//! }
//! ```

pub mod store;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, header, HeaderValue},
    response::Response,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::context::AppState;

type HmacSha256 = Hmac<Sha256>;

/// Session 配置
#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfig {
    /// 签名密钥（至少 32 字符）
    pub secret: String,
    /// Cookie 名称
    #[serde(default = "default_cookie_name")]
    pub cookie_name: String,
    /// 有效期（秒）
    #[serde(default = "default_max_age")]
    pub max_age: u64,
    /// HttpOnly 标记
    #[serde(default = "default_true")]
    pub http_only: bool,
    /// Secure 标记
    #[serde(default = "default_false")]
    pub secure: bool,
    /// SameSite 策略: strict / lax / none
    #[serde(default = "default_same_site")]
    pub same_site: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            secret: "arcx-default-secret-change-me-in-prod!!".to_string(),
            cookie_name: default_cookie_name(),
            max_age: default_max_age(),
            http_only: true,
            secure: false,
            same_site: default_same_site(),
        }
    }
}

fn default_cookie_name() -> String { "arcx.sid".to_string() }
fn default_max_age() -> u64 { 86400 }
fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_same_site() -> String { "lax".to_string() }

/// Session 数据（请求生命周期内可读写）
#[derive(Clone)]
pub struct Session {
    data: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    modified: Arc<Mutex<bool>>,
    config: Arc<SessionConfig>,
}

impl Session {
    /// 创建空 Session
    fn new(config: Arc<SessionConfig>) -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            modified: Arc::new(Mutex::new(false)),
            config,
        }
    }

    /// 从已有数据恢复
    fn from_data(data: HashMap<String, serde_json::Value>, config: Arc<SessionConfig>) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
            modified: Arc::new(Mutex::new(false)),
            config,
        }
    }

    /// 获取值
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let data = self.data.lock().unwrap();
        data.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 设置值
    pub fn set<T: Serialize>(&self, key: &str, value: T) {
        let mut data = self.data.lock().unwrap();
        if let Ok(v) = serde_json::to_value(value) {
            data.insert(key.to_string(), v);
            *self.modified.lock().unwrap() = true;
        }
    }

    /// 删除值
    pub fn remove(&self, key: &str) {
        let mut data = self.data.lock().unwrap();
        if data.remove(key).is_some() {
            *self.modified.lock().unwrap() = true;
        }
    }

    /// 清空 Session
    pub fn clear(&self) {
        let mut data = self.data.lock().unwrap();
        if !data.is_empty() {
            data.clear();
            *self.modified.lock().unwrap() = true;
        }
    }

    /// 是否被修改
    pub fn is_modified(&self) -> bool {
        *self.modified.lock().unwrap()
    }

    /// 序列化 session 数据为签名 cookie 值
    fn to_cookie_value(&self) -> Option<String> {
        let data = self.data.lock().unwrap();
        if data.is_empty() {
            return None;
        }
        let json = serde_json::to_string(&*data).ok()?;
        let encoded = URL_SAFE_NO_PAD.encode(json.as_bytes());
        let signature = sign(&encoded, &self.config.secret);
        Some(format!("{}.{}", encoded, signature))
    }

    /// 从签名 cookie 值解析
    fn from_cookie_value(value: &str, config: &Arc<SessionConfig>) -> Option<Self> {
        let parts: Vec<&str> = value.rsplitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }
        let signature = parts[0];
        let encoded = parts[1];

        // 验签
        if !verify(encoded, signature, &config.secret) {
            tracing::warn!("Session cookie signature mismatch");
            return None;
        }

        // 解码
        let json_bytes = URL_SAFE_NO_PAD.decode(encoded).ok()?;
        let data: HashMap<String, serde_json::Value> =
            serde_json::from_slice(&json_bytes).ok()?;

        Some(Self::from_data(data, config.clone()))
    }

    /// 构建 Set-Cookie 头
    fn build_set_cookie_header(&self) -> Option<String> {
        let value = self.to_cookie_value()?;
        let mut cookie = format!(
            "{}={}; Path=/; Max-Age={}",
            self.config.cookie_name, value, self.config.max_age
        );
        if self.config.http_only {
            cookie.push_str("; HttpOnly");
        }
        if self.config.secure {
            cookie.push_str("; Secure");
        }
        cookie.push_str(&format!("; SameSite={}", capitalize(&self.config.same_site)));
        Some(cookie)
    }

    /// 构建清除 Cookie 头
    fn build_clear_cookie_header(&self) -> String {
        format!(
            "{}=; Path=/; Max-Age=0; HttpOnly",
            self.config.cookie_name
        )
    }
}

/// Session 中间件层
/// 自动从 Cookie 中提取 Session，响应时自动写回
pub struct SessionLayer;

impl SessionLayer {
    /// 将 Session 变化写入响应
    pub fn finalize_response(session: &Session, mut response: Response) -> Response {
        if !session.is_modified() {
            return response;
        }

        let data = session.data.lock().unwrap();
        let header_value = if data.is_empty() {
            // 数据清空 → 清除 cookie
            session.build_clear_cookie_header()
        } else {
            drop(data);
            match session.build_set_cookie_header() {
                Some(v) => v,
                None => return response,
            }
        };

        if let Ok(val) = HeaderValue::from_str(&header_value) {
            response.headers_mut().append(header::SET_COOKIE, val);
        }

        response
    }
}

/// 实现 FromRequestParts，Session 可直接作为 handler 参数
#[axum::async_trait]
impl<S> FromRequestParts<S> for Session
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // 获取 session config（从 AppState 的资源池中）
        let config = app_state
            .resource::<SessionConfig>()
            .unwrap_or_else(|| Arc::new(SessionConfig::default()));

        // 从 Cookie 头解析已有 session
        let session = parts
            .headers
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| extract_cookie(cookies, &config.cookie_name))
            .and_then(|value| Session::from_cookie_value(&value, &config))
            .unwrap_or_else(|| Session::new(config));

        Ok(session)
    }
}

/// 从 Cookie 头中提取指定名称的值
fn extract_cookie(cookies: &str, name: &str) -> Option<String> {
    for pair in cookies.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix(&format!("{}=", name)) {
            return Some(value.to_string());
        }
    }
    None
}

/// HMAC-SHA256 签名
fn sign(data: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    URL_SAFE_NO_PAD.encode(result.into_bytes())
}

/// 验证签名
fn verify(data: &str, signature: &str, secret: &str) -> bool {
    let expected = sign(data, secret);
    // 常量时间比较防止时序攻击
    constant_time_eq(expected.as_bytes(), signature.as_bytes())
}

/// 常量时间字节比较
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// 首字母大写
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
