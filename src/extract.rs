//! 参数提取与校验
//! 
//! 框架提供 ValidJson<T> 提取器，自动解析 + 校验请求体。
//! 相比直接用 axum::Json<T>，ValidJson 会在反序列化后自动执行 validator 校验。
//! 
//! 用法：
//! ```rust
//! use validator::Validate;
//! 
//! #[derive(Deserialize, Validate)]
//! struct CreateUserDto {
//!     #[validate(length(min = 1, message = "name 不能为空"))]
//!     name: String,
//!     #[validate(email(message = "email 格式不正确"))]
//!     email: String,
//! }
//! 
//! async fn create(ctx: Context, body: ValidJson<CreateUserDto>) -> AppResult<...> {
//!     let dto = body.into_inner();
//!     // dto 已通过校验，可以安全使用
//! }
//! ```

use axum::{
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;

/// 带校验的 JSON 提取器
/// 自动执行: 反序列化 → validator 校验 → 返回结果
pub struct ValidJson<T>(pub T);

impl<T> ValidJson<T> {
    /// 获取内部值
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// 实现 FromRequest，使 ValidJson<T> 能作为 handler 参数
#[axum::async_trait]
impl<S, T> FromRequest<S> for ValidJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 先解析 JSON
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| {
                AppError::BadRequest(format!("请求体解析失败: {}", e)).into_response()
            })?;

        // 再执行校验
        value.validate().map_err(|e| {
            // 把 validator 的错误转为可读的消息
            let messages: Vec<String> = e
                .field_errors()
                .into_iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(move |err| {
                        err.message
                            .as_ref()
                            .map(|m| format!("{}: {}", field, m))
                            .unwrap_or_else(|| format!("{}: 校验失败", field))
                    })
                })
                .collect();

            AppError::BadRequest(messages.join("; ")).into_response()
        })?;

        Ok(ValidJson(value))
    }
}
