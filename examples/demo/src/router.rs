//! 路由声明 — 自由组合，想怎么写怎么写

use arcx_core::prelude::*;
use crate::controller;

pub fn routes(r: &mut ArcxRouter) {
    // 自由声明路由
    r.get("/api/home", controller::home::index);
    r.get("/api/home/:id", controller::home::show);
    r.post("/api/home", controller::home::create);
    r.put("/api/home/:id", controller::home::update);
    r.delete("/api/home/:id", controller::home::destroy);

    // 单独路由
    r.get("/api/health", controller::health::check);
}
