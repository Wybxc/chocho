//! 好友操作。
//!
//! `Friend` 是一个好友操作对象，提供了发送消息、发送语音、撤回消息、戳一戳、删除好友等操作。
//!
//! # Example
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

use std::time::Duration;

use chocho_msg::{elem::FriendImage, Message};
use ricq::{
    structs::{FriendAudio, LinkShare, MessageReceipt, MusicShare, MusicVersion},
    Client, RQResult,
};

/// 好友操作对象。
pub struct Friend<'a> {
    /// 客户端引用。
    pub client: &'a Client,
    /// 好友 QQ 号。
    pub uin: i64,
}

impl<'a> Friend<'a> {
    /// 发送消息。
    pub async fn send(&self, msg: impl Into<Message>) -> RQResult<MessageReceipt> {
        let msg: Message = msg.into();
        self.client.send_friend_message(self.uin, msg.into()).await
    }

    /// 上传语音。
    pub async fn upload_audio(
        &self,
        audio: impl AsRef<[u8]>,
        duration: Duration,
    ) -> RQResult<FriendAudio> {
        self.client
            .upload_friend_audio(self.uin, audio.as_ref(), duration)
            .await
    }

    /// 发送语音。
    pub async fn send_audio(&self, audio: FriendAudio) -> RQResult<MessageReceipt> {
        self.client.send_friend_audio(self.uin, audio).await
    }

    /// 获取语音下载链接。
    pub async fn get_audio_url(&self, audio: FriendAudio) -> RQResult<String> {
        self.client.get_friend_audio_url(self.uin, audio).await
    }

    /// 撤回消息。
    pub async fn recall(&self, receipt: MessageReceipt) -> RQResult<()> {
        self.client
            .recall_friend_message(self.uin, receipt.time, receipt.seqs, receipt.rands)
            .await
    }

    /// 上传图片。
    pub async fn upload_image(&self, image: impl AsRef<[u8]>) -> RQResult<FriendImage> {
        self.client
            .upload_friend_image(self.uin, image.as_ref())
            .await
    }

    /// 发送链接分享。
    pub async fn share_link(&self, link: LinkShare) -> RQResult<()> {
        self.client.send_friend_link_share(self.uin, link).await
    }

    /// 发送音乐分享。
    pub async fn share_music(&self, music: MusicShare, version: MusicVersion) -> RQResult<()> {
        self.client
            .send_friend_music_share(self.uin, music, version)
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
