//! JWT 鉴权插件
//! 
//! 配置方式：
//! ```toml
//! [plugin.jwt]
//! enable = true
//! secret = "your-secret-key"
//! expire = 86400        # token 有效期（秒），默认 24 小时
//! ```
//! 
//! 使用方式：
//! ```rust
//! // 生成 token
//! let jwt = ctx.resource::<JwtService>().unwrap();
//! let token = jwt.sign(claims)?;
//! 
//! // 路由守卫自动验证（见 guard 模块）
//! ```

use std::sync::Arc;
use std::any::Any;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use crate::plugin::{Plugin, PluginError};

/// JWT 配置
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expire: u64,
}

/// JWT 服务 —— 提供 sign / verify 方法
#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
}

/// JWT Claims 标准结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// 用户标识
    pub sub: String,
    /// 过期时间（Unix 时间戳）
    pub exp: usize,
    /// 签发时间
    pub iat: usize,
    /// 可选的额外数据
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// 签发 token
    pub fn sign(&self, sub: &str, data: Option<serde_json::Value>) -> Result<String, String> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: sub.to_string(),
            exp: now + self.config.expire as usize,
            iat: now,
            data,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_bytes()),
        )
        .map_err(|e| format!("JWT sign failed: {}", e))
    }

    /// 验证并解析 token
    pub fn verify(&self, token: &str) -> Result<Claims, String> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| format!("JWT verify failed: {}", e))
    }

    /// 获取配置的过期时间
    pub fn expire_seconds(&self) -> u64 {
        self.config.expire
    }
}

/// JWT 插件
pub struct JwtPlugin {
    service: Option<JwtService>,
}

impl JwtPlugin {
    pub fn new() -> Self {
        Self { service: None }
    }
}

#[async_trait::async_trait]
impl Plugin for JwtPlugin {
    fn name(&self) -> &str {
        "jwt"
    }

    async fn init(&mut self, config: &toml::Value) -> Result<(), PluginError> {
        let secret = config
            .get("secret")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::new("jwt", "Missing 'secret' in [plugin.jwt]"))?
            .to_string();

        let expire = config
            .get("expire")
            .and_then(|v| v.as_integer())
            .unwrap_or(86400) as u64;

        let jwt_config = JwtConfig { secret, expire };
        self.service = Some(JwtService::new(jwt_config));

        tracing::info!("    JWT plugin ready (expire: {}s)", expire);
        Ok(())
    }

    fn resource(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.service
            .as_ref()
            .map(|s| Arc::new(s.clone()) as Arc<dyn Any + Send + Sync>)
    }
}
