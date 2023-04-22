//! # chocho_macros
//!
//! [chocho](https://github.com/Wybxc/chocho) 的过程宏支持。
#![deny(missing_docs)]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{meta::ParseNestedMeta, parse_macro_input, Expr, ItemFn};

/// 声明 `chocho` 的主函数。
///
/// 主函数的签名为：
///
/// ```,no_run
/// # use chocho::prelude::*;
/// #[chocho::main]
/// async fn main(client: RQClient) {
///     // ...
/// }
/// ```
///
/// 该函数会在 `chocho` 启动并登录成功后被调用。
///
/// # 生命周期
///
/// `chocho` 的生命周期分为三个阶段：
///
/// 1. 初始化 `tracing-subscriber` 的日志输出，登录账号；
/// 2. 执行主函数。
/// 3. 开始自动断线重连。
///
/// # Attributes
///
/// - `data_folder`：指定 `chocho` 的数据文件夹路径。默认为 `./bots`。
/// - `handler`：指定 `chocho` 的事件处理器。默认为 `chocho::ricq::handler::DefaultHandler`。
///
/// 可以用以下语法指定属性：
/// ```,no_run
/// # use chocho::prelude::*;
/// # struct MyHandler;
/// # impl chocho::ricq::handler::PartlyHandler for MyHandler {}
/// #[chocho::main(data_folder = "./data", handler = MyHandler)]
/// async fn main(client: RQClient) {
///     // ...
/// }
/// ```
///
/// 或者
///
/// ```,no_run
/// # use chocho::prelude::*;
/// # struct MyHandler;
/// # impl chocho::ricq::handler::PartlyHandler for MyHandler {}
/// #[chocho::main]
/// #[chocho(data_folder = "./data", handler = MyHandler)]
/// async fn main(client: RQClient) {
///     // ...
/// }
/// ```
///
/// 或者
///
/// ```,no_run
/// # use chocho::prelude::*;
/// # struct MyHandler;
/// # impl chocho::ricq::handler::PartlyHandler for MyHandler {}
/// #[chocho::main]
/// #[chocho(data_folder = "./data")]
/// #[chocho(handler = MyHandler)]
/// async fn main(client: RQClient) {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        attrs, sig, block, ..
    } = parse_macro_input!(input as ItemFn);

    if sig.asyncness.is_none() {
        panic!("`#[main]` must be applied to an `async fn`");
    }
    if sig.constness.is_some() || sig.unsafety.is_some() {
        panic!("`#[main]` must not be applied to a `const fn` or `unsafe fn`");
    }
    if sig.abi.is_some() {
        panic!("`#[main]` must not be applied to an `extern fn`");
    }

    let mut data_folder = quote! { "./bots".to_string() };
    let mut handler = quote! { ::chocho::ricq::handler::DefaultHandler };
    let mut uin = quote! { ::std::option::Option::None };
    let mut login_method = quote! { ::std::option::Option::None };

    let mut meta_parser = |meta: ParseNestedMeta| {
        if meta.path.is_ident("data_folder") {
            let value: Expr = meta.value()?.parse()?;
            data_folder = quote! { ::std::string::String::from(#value) };
        } else if meta.path.is_ident("handler") {
            let value: Expr = meta.value()?.parse()?;
            handler = quote! { #value };
        } else if meta.path.is_ident("uin") {
            let value: Expr = meta.value()?.parse()?;
            uin = quote! { ::std::option::Option::Some(#value) };
        } else if meta.path.is_ident("login_method") {
            let value: Expr = meta.value()?.parse()?;
            login_method = quote! { ::std::option::Option::Some(#value) };
        } else {
            return Err(meta.error(format!(
                "unexpected attribute `{}`",
                meta.path.to_token_stream()
            )));
        }
        Ok(())
    };

    if !args.is_empty() {
        let arg_parser = syn::meta::parser(&mut meta_parser);
        parse_macro_input!(args with arg_parser);
    }
    for attr in attrs {
        if attr.path().is_ident("chocho") {
            attr.parse_nested_meta(&mut meta_parser).unwrap();
        }
    }

    let ident = sig.ident;
    let args = sig.inputs;
    let output = sig.output;

    let result = quote! {
        mod __chocho_private {
            pub(super) fn run<T>(
                fut: impl ::std::future::Future<
                    Output = ::std::result::Result<T, ::std::boxed::Box<dyn ::std::error::Error>>
                >
            ) -> ::std::result::Result<T, Box<dyn ::std::error::Error>> {
                ::chocho::tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed building the Runtime")
                    .block_on(fut)
            }

            pub(super) trait Wrap<T> {
                type Error;
                fn wrap(self) -> ::std::result::Result<T, Self::Error>;
            }

            impl<T> Wrap<T> for T {
                type Error = ::std::convert::Infallible;
                fn wrap(self) -> ::std::result::Result<T, Self::Error> {
                    Ok(self)
                }
            }

            impl<T, U> Wrap<T> for ::std::result::Result<T, U> {
                type Error = U;
                fn wrap(self) -> ::std::result::Result<T, U> {
                    self
                }
            }
        }

        #[allow(unreachable_code)]
        fn main() -> impl ::std::process::Termination {
            __chocho_private::run(async {
                async fn #ident(#args) #output {
                    #block
                }
                ::chocho::tracing_subscriber::fmt::init();
                ::chocho::tokio::spawn(async {
                    ::chocho::tokio::signal::ctrl_c().await.unwrap();
                    ::chocho::lifespan::do_finalize().await;
                    ::std::process::exit(0);
                });
                let (client, alive) = ::chocho::login(#data_folder, #handler, #uin, #login_method).await?;
                let result = __chocho_private::Wrap::wrap(#ident(client).await)?;
                alive.auto_reconnect().await?;
                ::chocho::lifespan::do_finalize().await;
                Ok(result)
            })
        }
    };
    result.into()
}
