use crate::prelude::*;

#[service]
impl OrderService {
    pub async fn find_by_user(&self, user_id: &str) -> AppResult<Value> {
        Ok(json!([
            { "order_id": "ORD-001", "user_id": user_id, "amount": 99.9 },
            { "order_id": "ORD-002", "user_id": user_id, "amount": 199.0 },
        ]))
    }
}
