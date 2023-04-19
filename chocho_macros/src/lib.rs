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
/// ```rust,ignore
/// #[chocho::main]
/// async fn main(client: Arc<Client>) {
///    // ...
/// }
/// ```
///
/// 该函数会在 `chocho` 启动并登录成功后被调用。
///
/// 主函数执行完成后，`chocho` 接管程序生命周期，开始自动断线重连，接受并处理事件。
///
/// # Attributes
/// - `data_folder`：指定 `chocho` 的数据文件夹路径。默认为 `./bots`。
/// - `handler`：指定 `chocho` 的事件处理器。默认为 `chocho::ricq::handler::DefaultHandler`。
///
/// 可以用以下语法指定属性：
/// ```rust,ignore
/// #[chocho::main(data_folder = "./data", handler = MyHandler)]
/// async fn main(client: Arc<Client>) {
///     // ...
/// }
/// ```
///
/// 或者
///
/// ```rust,ignore
/// #[chocho::main]
/// #[chocho(data_folder = "./data", handler = MyHandler)]
/// async fn main(client: Arc<Client>) {
///     // ...
/// }
/// ```
///
/// 或者
///
/// ```rust,ignore
/// #[chocho::main]
/// #[chocho(data_folder = "./data")]
/// #[chocho(handler = MyHandler)]
/// async fn main(client: Arc<Client>) {
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
    let mut meta_parser = |meta: ParseNestedMeta| {
        if meta.path.is_ident("data_folder") {
            let value: Expr = meta.value()?.parse()?;
            data_folder = quote! { ::std::string::String::from(#value) };
        } else if meta.path.is_ident("handler") {
            let value: Expr = meta.value()?.parse()?;
            handler = quote! { #value };
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

        fn main() -> impl ::std::process::Termination {
            __chocho_private::run(async {
                async fn #ident(#args) #output {
                    #block
                }
                let (client, alive) = ::chocho::init(#data_folder, #handler).await?;
                let result = __chocho_private::Wrap::wrap(#ident(client).await)?;
                alive.auto_reconnect().await?;
                Ok(result)
            })
        }
    };
    result.into()
}
