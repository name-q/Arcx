#![allow(dead_code)]

use arcx_core::prelude::*;

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
        let token = parts.headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::unauthorized("Missing Authorization header"))?;

        if token.is_empty() {
            return Err(AppError::unauthorized("Empty token"));
        }

        Ok(AuthUser {
            id: "user_from_token".to_string(),
            payload: serde_json::json!({ "role": "admin" }),
        })
    }
}
