//! Order Service — 展示 Service 互调目标

use std::sync::Arc;
use arcx_core::prelude::*;

pub struct OrderService {
    ctx: Ctx,
}

impl Service for OrderService {
    fn create(ctx: &Ctx) -> Arc<Self> {
        Arc::new(Self { ctx: ctx.clone() })
    }
}

impl OrderService {
    /// 查询用户的订单
    pub async fn find_by_user(&self, user_id: &str) -> AppResult<Value> {
        // 可以通过 ctx 读取配置
        let _env = self.ctx.env();

        // 模拟查询
        Ok(json!([
            { "order_id": "ORD-001", "user_id": user_id, "amount": 99.9 },
            { "order_id": "ORD-002", "user_id": user_id, "amount": 199.0 },
        ]))
    }
}
