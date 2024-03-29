//! # chocho
//!
//! 一个基于 [ricq](https://docs.rs/ricq) 的 QQ 机器人框架。
//!
//! ## Example
//!
//! ```,no_run
//! use chocho::prelude::*;
//!
//! use async_trait::async_trait;
//! use chocho::ricq::handler::PartlyHandler;
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
//! async fn main(client: RQClient) {
//!     let account_info = client.account_info.read().await;
//!     tracing::info!("{:?}", account_info);
//! }
//! ```
#![deny(missing_docs)]

pub mod common;
pub mod lifespan;
pub mod prelude;

pub use chocho_client::{ClientExt, RQClient};
pub use chocho_login::{login, LoginMethod, QSignClient, RQProtocol};
pub use chocho_macros::main;
pub use chocho_msg::{Message, RQElem};
pub use lifespan::finalizer;
#[doc(hidden)]
pub use ricq;
#[doc(hidden)]
pub use tokio;
#[doc(hidden)]
pub use tracing_subscriber;
