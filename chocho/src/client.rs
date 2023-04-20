//! 客户端模块，提供了用于处理事件的客户端和客户端扩展方法。
//!
//! # Examples
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

use ricq::{
    structs::{FriendAudio, MessageReceipt},
    Client, RQResult,
};
use std::sync::Arc;

use crate::msg::Message;

/// `ricq` 客户端的别名。
pub type RQClient = Arc<Client>;

/// 客户端扩展方法。
#[async_trait::async_trait]
pub trait ClientExt {
    /// 进行好友操作。
    fn friend(&self, uin: i64) -> Friend;
}

impl ClientExt for Client {
    fn friend(&self, uin: i64) -> Friend {
        Friend { client: self, uin }
    }
}

/// 好友操作对象。
pub struct Friend<'a> {
    client: &'a Client,
    uin: i64,
}

impl<'a> Friend<'a> {
    /// 发送消息。
    pub async fn send(&self, msg: impl Into<Message>) -> RQResult<MessageReceipt> {
        let msg: Message = msg.into();
        self.client.send_friend_message(self.uin, msg.into()).await
    }

    /// 发送语音。
    pub async fn send_audio(&self, audio: FriendAudio) -> RQResult<MessageReceipt> {
        self.client.send_friend_audio(self.uin, audio).await
    }

    /// 撤回消息。
    pub async fn recall(&self, receipt: MessageReceipt) -> RQResult<()> {
        self.client
            .recall_friend_message(self.uin, receipt.time, receipt.seqs, receipt.rands)
            .await
    }

    /// 戳一戳。
    pub async fn poke(&self) -> RQResult<()> {
        self.client.friend_poke(self.uin).await
    }

    /// 删除好友。
    pub async fn delete(self) -> RQResult<()> {
        self.client.delete_friend(self.uin).await
    }
}
