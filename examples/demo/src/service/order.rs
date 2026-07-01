//! Order Service — 干净的业务逻辑

use arcx_core::prelude::*;

#[service]
impl OrderService {
    /// 查询用户的订单
    pub async fn find_by_user(&self, user_id: &str) -> AppResult<Value> {
        let _env = self.ctx.env();

        Ok(json!([
            { "order_id": "ORD-001", "user_id": user_id, "amount": 99.9 },
            { "order_id": "ORD-002", "user_id": user_id, "amount": 199.0 },
        ]))
    }
}
