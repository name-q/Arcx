//! 路由声明 — 自由组合

use arcx_core::prelude::*;
use crate::controller;

pub fn routes(r: &mut ArcxRouter) {
    // ─── 公开路由 ───────────────────────────
    r.get("/api/home", controller::home::index);
    r.get("/api/home/query", controller::home::query_demo);
    r.get("/api/home/:id", controller::home::show);
    r.get("/api/home/:id/detail", controller::home::detail);
    r.post("/api/home", controller::home::create);
    r.delete("/api/home/:id", controller::home::destroy);

    r.get("/api/health", controller::health::check);

    // ─── 鉴权路由 ───────────────────────────
    r.guarded_scope("/api/admin", |s| {
        s.get("/profile", controller::admin::profile);
        s.get("/dashboard", controller::admin::dashboard);
    });
}
