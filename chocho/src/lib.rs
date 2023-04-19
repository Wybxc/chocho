//! # chocho
//!
//! 一个基于 [ricq](https://docs.rs/ricq) 的 QQ 机器人框架。
//!
//! ## Example
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use async_trait::async_trait;
//! use chocho::ricq::{handler::PartlyHandler, Client};
//!
//! struct Handler;
//! #[async_trait]
//! impl PartlyHandler for Handler {
//!     async fn handle_login(&self, uin: i64) {
//!         tracing::info!("登录成功: {}", uin);
//!     }
//! }
//!
//! #[chocho::main(handler = Handler)]
//! async fn main(client: Arc<Client>) {
//!     let account_info = client.account_info.read().await;
//!     tracing::info!("{:?}", account_info);
//! }
//! ```
#![deny(missing_docs)]

use std::sync::Arc;

use anyhow::Result;
use login::AliveHandle;
use requestty::Question;
use ricq::{handler::Handler, Client};

mod device;
mod login;
mod utils;

pub use chocho_macros::main;
#[doc(hidden)]
pub use ricq;
#[doc(hidden)]
pub use tokio;

#[doc(hidden)]
pub async fn init(
    data_folder: String,
    handler: impl Handler + 'static + Send + Sync,
) -> Result<(Arc<Client>, AliveHandle)> {
    tracing_subscriber::fmt::init();

    let uin = Question::int("uin").message("请输入账号").build();
    let uin = requestty::prompt_one(uin)?.as_int().unwrap();

    let login_method = Question::select("login_method")
        .message("请选择登录方式：")
        .choice("密码登录")
        .choice("二维码登录")
        .build();
    let login_method = requestty::prompt_one(login_method)?
        .as_list_item()
        .unwrap()
        .index;
    match login_method {
        0 => {
            // 密码登录
            let protocol = Question::select("protocol")
                .message("请选择客户端协议：")
                .choice("IPad")
                .choice("Android Phone")
                .choice("Android Watch")
                .choice("MacOS")
                .choice("企点")
                .default(0)
                .build();
            let protocol = requestty::prompt_one(protocol)?
                .as_list_item()
                .unwrap()
                .index;
            let protocol = match protocol {
                0 => login::Protocol::IPad,
                1 => login::Protocol::AndroidPhone,
                2 => login::Protocol::AndroidWatch,
                3 => login::Protocol::MacOS,
                4 => login::Protocol::QiDian,
                _ => unreachable!(),
            };

            let password = Question::password("password")
                .message("请输入密码")
                .mask('*')
                .build();
            let password = requestty::prompt_one(password)?.try_into_string().unwrap();

            login::login_with_password(uin, &password, protocol, data_folder, handler).await
        }
        1 => {
            // 二维码登录
            login::login_with_qrcode(
                uin,
                |img| {
                    println!("{}", utils::qrcode_text(&img)?);
                    Ok(())
                },
                data_folder,
                handler,
            )
            .await
        }
        _ => unreachable!(),
    }
}
