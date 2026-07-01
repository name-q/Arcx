use crate::prelude::*;
use super::order::OrderService;

#[service]
impl UserService {
    pub async fn get_profile(&self, user_id: &str) -> AppResult<Value> {
        Ok(json!({
            "user_id": user_id,
            "nickname": "Arcx User",
            "level": "pro",
        }))
    }

    pub async fn get_user_with_orders(&self, user_id: &str) -> AppResult<Value> {
        let profile = self.get_profile(user_id).await?;
        let order_svc = self.ctx.service::<OrderService>();
        let orders = order_svc.find_by_user(user_id).await?;

        Ok(json!({
            "user": profile,
            "orders": orders,
        }))
    }
}
