use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// 框架统一错误类型
/// 所有 Controller 返回 Result<T, AppError> 即可自动转为 JSON 错误响应
#[derive(Debug)]
pub enum AppError {
    /// 资源未找到 (404)
    NotFound(String),
    /// 参数校验失败 (400)
    BadRequest(String),
    /// 未授权 (401)
    Unauthorized(String),
    /// 内部错误 (500)
    Internal(String),
}

/// 实现 IntoResponse，让 Axum 自动将 AppError 转为 HTTP 响应
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = json!({
            "success": false,
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

/// 统一成功响应包装
pub fn success<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": data
    }))
}

/// 类型别名，Controller 统一返回类型
pub type AppResult<T> = Result<T, AppError>;
