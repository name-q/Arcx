//! User Service — 干净的业务逻辑，无胶水代码

use arcx_core::prelude::*;
use super::order::OrderService;

#[service]
impl UserService {
    /// 获取用户资料
    pub async fn get_profile(&self, user_id: &str) -> AppResult<Value> {
        Ok(json!({
            "user_id": user_id,
            "nickname": "Arcx User",
            "level": "pro",
        }))
    }

    /// 获取用户及其订单（展示 Service 互调）
    pub async fn get_user_with_orders(&self, user_id: &str) -> AppResult<Value> {
        let profile = self.get_profile(user_id).await?;

        // Service 互调：通过 self.ctx（由 #[service] 宏自动生成）
        let order_svc = self.ctx.service::<OrderService>();
        let orders = order_svc.find_by_user(user_id).await?;

        Ok(json!({
            "user": profile,
            "orders": orders,
        }))
    }
}
