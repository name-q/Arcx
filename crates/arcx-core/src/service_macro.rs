//! `services!` 声明式宏 — 自动生成 Services Container + Ctx 扩展

/// `services!` — 一行注册所有 Service
///
/// 生成模块声明、Services 容器、ServiceAccess trait。
/// Controller 通过 `ctx.services().user.xxx()` 调用。
///
/// ## 用法
///
/// ```rust
/// // src/service/mod.rs
/// arcx::services! {
///     user: UserService,
///     order: OrderService,
/// }
/// ```
///
/// ```rust
/// // src/controller/home.rs
/// use crate::service::ServiceAccess;
///
/// pub async fn show(ctx: Ctx) -> AppResult<impl IntoResponse> {
///     let user = ctx.services().user.find_by_id(42).await?;
///     Ok(response::success(user))
/// }
/// ```
#[macro_export]
macro_rules! services {
    ( $( $field:ident : $service_type:ident ),* $(,)? ) => {
        // 模块声明
        $(
            pub mod $field;
        )*

        /// 自动生成的 Services 容器
        ///
        /// 每个字段是 `Arc<XxxService>`，通过 Ctx 的请求级缓存获取。
        /// 同一请求内多次访问同一 Service 返回相同实例。
        pub struct Services {
            $(
                pub $field: ::std::sync::Arc<$field::$service_type>,
            )*
        }

        impl Services {
            /// 从 Ctx 构造 — 内部走 ctx.service::<T>() 缓存
            pub fn new(ctx: &$crate::ctx::Ctx) -> Self {
                Self {
                    $(
                        $field: ctx.service::<$field::$service_type>(),
                    )*
                }
            }
        }

        /// Ctx 扩展 — 让 ctx.services() 返回带名字段的容器
        pub trait ServiceAccess {
            fn services(&self) -> Services;
        }

        impl ServiceAccess for $crate::ctx::Ctx {
            fn services(&self) -> Services {
                Services::new(self)
            }
        }
    };
}
