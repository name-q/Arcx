use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// 框架统一错误类型
/// Controller 返回 AppResult<T> 即可自动转为规范化的 JSON 错误响应
///
/// 约定响应格式：
/// 成功: { "success": true, "data": ... }
/// 失败: { "success": false, "error": { "code": 404, "message": "..." } }
#[derive(Debug)]
pub enum AppError {
    /// 400 - 参数错误
    BadRequest(String),
    /// 401 - 未授权
    Unauthorized(String),
    /// 403 - 禁止访问
    Forbidden(String),
    /// 404 - 资源不存在
    NotFound(String),
    /// 500 - 内部错误
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
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

/// 成功响应包装
pub fn success<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": data
    }))
}

/// Controller 统一返回类型
pub type AppResult<T> = Result<T, AppError>;
