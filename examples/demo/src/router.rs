//! 路由声明
//!
//! 集中声明所有路由，一目了然。

use arcx_core::prelude::*;
use crate::controller;

pub fn routes(r: &mut ArcxRouter) {
    // RESTful 资源路由
    r.resources("/api/home", controller::home::handlers());

    // 单独路由
    r.get("/api/health", controller::health::check);
}
