//! Admin Controller — guarded_scope 保护的路由示例

use arcx_core::prelude::*;
use crate::helper;

/// GET /api/admin/profile — 需要登录才能访问
pub async fn profile(_ctx: Context, user: AuthUser) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({
        "id": user.id,
        "role": user.payload["role"],
        "message": "This is a protected route"
    })))
}

/// GET /api/admin/dashboard — 需要登录
pub async fn dashboard(_ctx: Context, user: AuthUser) -> AppResult<impl IntoResponse> {
    Ok(helper::success(json!({
        "user_id": user.id,
        "stats": { "total_users": 42, "active_today": 7 }
    })))
}
