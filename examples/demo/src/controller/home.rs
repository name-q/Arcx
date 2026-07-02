use crate::prelude::*;

/// GET /api/home — 展示 Ctx 快捷方法
pub async fn index(ctx: Ctx) -> AppResult<impl IntoResponse> {
    let name = &ctx.config().app.name;
    let version = &ctx.config().app.version;
    let port: Option<u16> = ctx.conf("server.port");
    let custom: Option<String> = ctx.conf("custom.greeting");

    // 新增：请求元信息快捷方法
    let method = ctx.method().to_string();
    let path = ctx.path().to_string();
    let uri = ctx.uri().to_string();
    let ip = ctx.ip().map(|ip| ip.to_string());
    let user_agent = ctx.header("user-agent").map(|s| s.to_string());

    Ok(Json(json!({
        "app": {
            "name": name,
            "version": version,
            "port": port,
            "greeting": custom.unwrap_or_else(|| "Hello from Arcx!".into()),
        },
        "request": {
            "method": method,
            "path": path,
            "uri": uri,
            "ip": ip,
            "user_agent": user_agent,
        }
    })))
}

/// GET /api/home/query — 展示 ctx.query::<T>()
#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_size")]
    pub size: u32,
}
fn default_page() -> u32 { 1 }
fn default_size() -> u32 { 20 }

pub async fn query_demo(ctx: Ctx) -> AppResult<impl IntoResponse> {
    let pagination = ctx.query::<Pagination>()?;
    Ok(Json(json!({
        "page": pagination.page,
        "size": pagination.size,
        "path": ctx.path(),
    })))
}

/// GET /api/home/:id — ctx.services().user 链式调用
pub async fn show(ctx: Ctx, Path(id): Path<String>) -> AppResult<impl IntoResponse> {
    let user = ctx.services().user.get_profile(&id).await?;
    Ok(response::success(user))
}

/// GET /api/home/:id/detail — Service 互调
pub async fn detail(ctx: Ctx, Path(id): Path<String>) -> AppResult<impl IntoResponse> {
    let data = ctx.services().user.get_user_with_orders(&id).await?;
    Ok(response::success(data))
}

/// POST /api/home
pub async fn create(Json(body): Json<Value>) -> AppResult<impl IntoResponse> {
    Ok(response::created(json!({ "item": body })))
}

/// DELETE /api/home/:id
pub async fn destroy(Path(_id): Path<u64>) -> AppResult<impl IntoResponse> {
    Ok(response::no_content())
}
