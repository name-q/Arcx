//! Arcx Procedural Macros
//!
//! 提供 `#[service]` 属性宏，自动生成 Service 胶水代码。

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, Type};

/// `#[service]` — 消除 Service 胶水代码
///
/// 标注在 `impl XxxService { ... }` 上，自动生成：
/// - `pub struct XxxService { ctx: arcx_core::ctx::Ctx }`
/// - `impl arcx_core::ctx::Service for XxxService { fn create(...) }`
///
/// ## 用法
///
/// ```rust
/// use arcx_core::prelude::*;
///
/// #[service]
/// impl UserService {
///     pub async fn find_by_id(&self, id: u64) -> AppResult<Value> {
///         Ok(json!({ "id": id, "name": format!("User_{}", id) }))
///     }
///
///     pub async fn find_with_orders(&self, id: u64) -> AppResult<Value> {
///         let user = self.find_by_id(id).await?;
///         // self.ctx 自动可用
///         let orders = self.ctx.service::<OrderService>().find_by_user(id).await?;
///         Ok(json!({ "user": user, "orders": orders }))
///     }
/// }
/// ```
///
/// 展开后等价于：
///
/// ```rust
/// pub struct UserService {
///     pub(crate) ctx: arcx_core::ctx::Ctx,
/// }
///
/// impl arcx_core::ctx::Service for UserService {
///     fn create(ctx: &arcx_core::ctx::Ctx) -> std::sync::Arc<Self> {
///         std::sync::Arc::new(Self { ctx: ctx.clone() })
///     }
/// }
///
/// impl UserService {
///     // ...用户写的方法
/// }
/// ```
#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    // 提取类型名（只支持简单路径如 UserService）
    let self_ty = &input.self_ty;
    let struct_name = match self_ty.as_ref() {
        Type::Path(type_path) => {
            let seg = type_path.path.segments.last().expect("#[service] requires a type name");
            seg.ident.clone()
        }
        _ => panic!("#[service] can only be applied to `impl TypeName {{ ... }}`"),
    };

    // 保留 impl 块原封不动
    let impl_block = &input;

    let expanded = quote! {
        pub struct #struct_name {
            pub(crate) ctx: arcx_core::ctx::Ctx,
        }

        impl arcx_core::ctx::Service for #struct_name {
            fn create(ctx: &arcx_core::ctx::Ctx) -> ::std::sync::Arc<Self> {
                ::std::sync::Arc::new(Self { ctx: ctx.clone() })
            }
        }

        #impl_block
    };

    TokenStream::from(expanded)
}
