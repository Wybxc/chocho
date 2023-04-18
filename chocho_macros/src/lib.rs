use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
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

    let ident = sig.ident;
    let args = sig.inputs;
    let output = sig.output;

    let mut data_folder = quote! {
        "./bots".to_string()
    };

    for attr in attrs {
        let attr = attr.meta.require_name_value().unwrap();
        match attr
            .path
            .get_ident()
            .unwrap_or_else(|| {
                panic!(
                    "expected identifier, found `{}`",
                    attr.path.to_token_stream()
                )
            })
            .to_string()
            .as_str()
        {
            "data_folder" => {
                let value = &attr.value;
                data_folder = quote! {
                    ::std::string::String::from(#value)
                };
            }
            _ => panic!("unexpected attribute `{}`", attr.path.to_token_stream()),
        }
    }

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
                let (client, alive) = ::chocho::init(#data_folder).await?;
                let result = __chocho_private::Wrap::wrap(#ident(client).await)?;
                alive.auto_reconnect().await?;
                Ok(result)
            })
        }
    };
    result.into()
}
