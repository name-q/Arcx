use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// 字段校验错误详情
#[derive(Debug, Clone, serde::Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    pub code: String,
}

/// 框架统一错误类型
///
/// Controller / Service 通过 `Err(AppError::xxx(...))` 抛出异常，
/// 框架全局错误中间件统一捕获并格式化为 JSON 响应。
///
/// ## 响应格式（默认）
///
/// ```json
/// { "error": "错误描述", "detail": [...] }  // detail 可选
/// ```
///
/// ## 状态码映射
///
/// - BadRequest    → 400
/// - Unauthorized  → 401
/// - Forbidden     → 403
/// - NotFound      → 404
/// - Validation    → 422
/// - Internal      → 500
#[derive(Debug)]
pub enum AppError {
    /// 400 - 请求参数错误
    BadRequest(String),
    /// 401 - 未授权
    Unauthorized(String),
    /// 403 - 禁止访问
    Forbidden(String),
    /// 404 - 资源不存在
    NotFound(String),
    /// 422 - 参数校验失败
    Validation {
        message: String,
        detail: Vec<FieldError>,
    },
    /// 500 - 内部错误
    Internal(String),
}

impl AppError {
    /// 400 Bad Request
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    /// 401 Unauthorized
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    /// 403 Forbidden
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    /// 404 Not Found
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// 422 Validation Failed
    pub fn validation(detail: Vec<FieldError>) -> Self {
        Self::Validation {
            message: "Validation Failed".to_string(),
            detail,
        }
    }

    /// 500 Internal Server Error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                json!({ "error": msg }),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                json!({ "error": msg }),
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                json!({ "error": msg }),
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                json!({ "error": msg }),
            ),
            AppError::Validation { message, detail } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "error": message, "detail": detail }),
            ),
            AppError::Internal(msg) => {
                // 生产环境不暴露内部错误细节
                let is_prod = std::env::var("ARCX_ENV")
                    .unwrap_or_default()
                    .eq_ignore_ascii_case("prod");
                let error_msg = if is_prod {
                    "Internal Server Error".to_string()
                } else {
                    msg
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": error_msg }),
                )
            }
        };

        (status, Json(body)).into_response()
    }
}

/// Controller 统一返回类型
pub type AppResult<T> = Result<T, AppError>;
