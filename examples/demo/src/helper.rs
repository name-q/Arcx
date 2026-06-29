//! 响应 Helper — 项目的响应格式约定，按需修改
#![allow(dead_code)]
//!
//! 这是用户代码，框架不依赖它。你可以：
//! - 修改 success 的 JSON 结构
//! - 添加自己的响应方法
//! - 或者完全不用它，直接返回 axum 原生类型

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use serde_json::json;

/// 成功响应（200）
pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
    Json(json!({
        "code": 0,
        "data": data,
        "message": "success"
    }))
}

/// 成功响应 + 自定义消息
pub fn success_msg<T: Serialize>(data: T, msg: &str) -> impl IntoResponse {
    Json(json!({
        "code": 0,
        "data": data,
        "message": msg
    }))
}

/// 创建成功（201）
pub fn created<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({
        "code": 0,
        "data": data,
        "message": "created"
    })))
}

/// 无内容（204）
pub fn no_content() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

/// 分页响应
pub fn paginate<T: Serialize>(list: Vec<T>, total: u64, page: u64, page_size: u64) -> impl IntoResponse {
    Json(json!({
        "code": 0,
        "data": {
            "list": list,
            "total": total,
            "page": page,
            "page_size": page_size
        }
    }))
}

/// 业务失败（200 但 code != 0）
pub fn fail(code: i32, msg: &str) -> impl IntoResponse {
    Json(json!({
        "code": code,
        "message": msg
    }))
}
