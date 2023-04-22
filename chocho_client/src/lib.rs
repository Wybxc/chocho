//! # chocho_client
//!
//! [chocho](https://github.com/Wybxc/chocho) 的客户端模块，提供了用于处理事件的客户端和客户端扩展方法。
//!
//! ## Examples
//!
//! ```no_run
//! use chocho::prelude::*;
//!
//! #[chocho::main]
//! async fn main(client: RQClient) -> RQResult<()> {
//!     client.friend(12345678).send("你好".to_string()).await?;
//!     Ok(())
//! }
//! ```
#![deny(missing_docs)]

pub mod friend;

use friend::Friend;

/// `ricq` 客户端的别名。
pub type RQClient = std::sync::Arc<ricq::Client>;

/// 客户端扩展方法。
#[async_trait::async_trait]
pub trait ClientExt {
    /// 进行好友操作。
    fn friend(&self, uin: i64) -> Friend;
}

impl ClientExt for ricq::Client {
    fn friend(&self, uin: i64) -> Friend {
        Friend { client: self, uin }
    }
}
