//! 路由声明 — 自由组合

use arcx_core::prelude::*;
use crate::controller;

pub fn routes(r: &mut ArcxRouter) {
    // ─── 公开路由 ───────────────────────────
    r.get("/api/home", controller::home::index);
    r.get("/api/home/:id", controller::home::show);
    r.post("/api/home", controller::home::create);
    r.put("/api/home/:id", controller::home::update);
    r.delete("/api/home/:id", controller::home::destroy);

    r.get("/api/health", controller::health::check);

    // ─── 鉴权路由（需要登录才能访问）───────────────────────────
    // 这组路由自动调用你注册的 AuthProvider::authenticate
    // 验证通过 → handler 中可提取 AuthUser
    // 验证失败 → 直接返回错误，不进 handler
    r.guarded_scope("/api/admin", |s| {
        s.get("/profile", controller::admin::profile);
        s.get("/dashboard", controller::admin::dashboard);
    });
}
