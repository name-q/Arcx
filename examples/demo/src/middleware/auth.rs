#![allow(dead_code)]
//! 鉴权实现 — 用户代码，按需修改
//!
//! 这里演示 JWT 方式。你可以替换为 Session、OAuth、远程验证等任何方案。

use arcx_core::prelude::*;

/// 你的鉴权提供者
pub struct JwtAuth {
    secret: String,
}

impl JwtAuth {
    pub fn new(secret: impl Into<String>) -> Self {
        Self { secret: secret.into() }
    }
}

#[async_trait]
impl AuthProvider for JwtAuth {
    async fn authenticate(&self, parts: &RequestParts) -> Result<AuthUser, AppError> {
        // 1. 从 Header 取 token（你可以改成从 cookie、query 等任意位置取）
        let token = parts.headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::unauthorized("Missing Authorization header"))?;

        // 2. 验证 token（这里用 JWT 插件，你也可以用任何验证方式）
        // 简化示例：直接 decode，实际项目请用 jsonwebtoken crate
        if token == "test-token" {
            // 模拟验证成功
            Ok(AuthUser {
                id: "user_001".to_string(),
                payload: json!({ "role": "admin" }),
            })
        } else {
            Err(AppError::unauthorized("Invalid or expired token"))
        }
    }
}
