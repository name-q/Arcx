use crate::prelude::*;
use crate::service::user::UserService;

/// GET /api/admin/profile — 需要登录 + Service 互调
pub async fn profile(ctx: Ctx, user: AuthUser) -> AppResult<impl IntoResponse> {
    let user_svc = ctx.service::<UserService>();
    let data = user_svc.get_user_with_orders(&user.id).await?;

    Ok(response::success(json!({
        "auth_user": { "id": user.id, "role": user.payload["role"] },
        "data": data,
    })))
}

/// GET /api/admin/dashboard
pub async fn dashboard(user: AuthUser) -> AppResult<impl IntoResponse> {
    Ok(response::success(json!({
        "user_id": user.id,
        "stats": { "total_users": 42, "active_today": 7 }
    })))
}
