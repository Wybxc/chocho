//! 群组操作。

use std::{collections::HashMap, time::Duration};

use chocho_msg::{
    elem::{Anonymous, GroupImage},
    Message,
};
use ricq::{
    structs::{
        GroupAudio, GroupInfo, GroupMemberInfo, GroupMemberPermission, LinkShare, MessageReceipt,
        MusicShare, MusicVersion,
    },
    Client, RQResult,
};
use ricq_core::command::oidb_svc::GroupAtAllRemainInfo;

use crate::structs::AudioCodeC;

/// 群组操作对象。
pub struct Group<'a> {
    /// 客户端引用。
    pub client: &'a Client,
    /// 群号。
    pub code: i64,
}

impl<'a> Group<'a> {
    /// 进行群成员操作。
    pub fn member(&self, uin: i64) -> GroupMember {
        GroupMember {
            client: self.client,
            code: self.code,
            uin,
        }
    }

    /// 发送消息。
    pub async fn send(&self, msg: impl Into<Message>) -> RQResult<MessageReceipt> {
        let msg: Message = msg.into();
        if msg.is_long() {
            self.client
                .send_group_long_message(self.code, msg.into())
                .await
        } else {
            self.client.send_group_message(self.code, msg.into()).await
        }
    }

    /// 获取群信息。
    pub async fn get_info(&self) -> RQResult<Option<GroupInfo>> {
        self.client.get_group_info(self.code).await
    }

    /// 获取群成员列表。
    pub async fn get_member_list(&self, owner: i64) -> RQResult<Vec<GroupMemberInfo>> {
        self.client.get_group_member_list(self.code, owner).await
    }

    /// 获取群主/管理员列表。
    pub async fn get_admin_list(&self) -> RQResult<HashMap<i64, GroupMemberPermission>> {
        self.client.get_group_admin_list(self.code).await
    }

    /// 上传语音。
    pub async fn upload_audio(
        &self,
        audio: impl AsRef<[u8]>,
        codec: AudioCodeC,
    ) -> RQResult<GroupAudio> {
        let codec = match codec {
            AudioCodeC::Amr => 0,
            AudioCodeC::Silk => 1,
        };
        self.client
            .upload_group_audio(self.code, audio.as_ref(), codec)
            .await
    }

    /// 发送语音。
    pub async fn send_audio(&self, audio: GroupAudio) -> RQResult<MessageReceipt> {
        self.client.send_group_audio(self.code, audio).await
    }

    /// 撤回消息。
    pub async fn recall(&self, receipt: MessageReceipt) -> RQResult<()> {
        self.client
            .recall_group_message(self.code, receipt.seqs, receipt.rands)
            .await
    }

    /// 上传图片。
    pub async fn upload_image(&self, image: impl AsRef<[u8]>) -> RQResult<GroupImage> {
        self.client
            .upload_group_image(self.code, image.as_ref())
            .await
    }

    /// 发送链接分享。
    pub async fn share_link(&self, link: LinkShare) -> RQResult<()> {
        self.client.send_group_link_share(self.code, link).await
    }

    /// 发送音乐分享。
    pub async fn share_music(&self, music: MusicShare, version: MusicVersion) -> RQResult<()> {
        self.client
            .send_group_music_share(self.code, music, version)
            .await
    }

    /// 戳一戳。
    pub async fn poke(&self, uin: i64) -> RQResult<()> {
        self.client.group_poke(self.code, uin).await
    }

    /// 退出群聊。
    pub async fn quit(self) -> RQResult<()> {
        self.client.group_quit(self.code).await
    }

    /// 设置群名称。
    pub async fn set_name(&self, name: impl Into<String>) -> RQResult<()> {
        self.client.update_group_name(self.code, name.into()).await
    }

    /// 设置群公告。
    pub async fn set_announcement(&self, announcement: impl Into<String>) -> RQResult<()> {
        self.client
            .update_group_memo(self.code, announcement.into())
            .await
    }

    /// 邀请入群。
    #[doc(hidden)]
    pub async fn invite(&self, uin: i64) -> RQResult<()> {
        self.client.group_invite(self.code, uin).await
    }

    /// 获取 @全体成员 剩余次数
    pub async fn get_at_all_remain(&self) -> RQResult<GroupAtAllRemainInfo> {
        self.client.group_at_all_remain(self.code).await
    }

    /// 获取自己的匿名信息，用于发送群消息。
    pub async fn get_anonymous(&self) -> RQResult<Option<Anonymous>> {
        self.client.get_anony_info(self.code).await
    }

    /// 群聊打卡。
    pub async fn clock_in(&self) -> RQResult<()> {
        self.client.group_sign_in(self.code).await
    }
}

/// 群成员操作对象。
pub struct GroupMember<'a> {
    /// 客户端引用。
    pub client: &'a Client,
    /// 群号。
    pub code: i64,
    /// 群成员 QQ 号。
    pub uin: i64,
}

impl<'a> GroupMember<'a> {
    /// 发送群成员临时消息。
    pub async fn send_temp_msg(&self, msg: impl Into<Message>) -> RQResult<MessageReceipt> {
        let msg: Message = msg.into();
        self.client
            .send_group_temp_message(self.code, self.uin, msg.into())
            .await
    }

    /// 获取群成员信息。
    pub async fn get_info(&self) -> RQResult<GroupMemberInfo> {
        self.client.get_group_member_info(self.code, self.uin).await
    }

    /// 禁言。
    pub async fn mute(&self, time: Duration) -> RQResult<()> {
        self.client.group_mute(self.code, self.uin, time).await
    }

    /// 解除禁言。
    pub async fn unmute(&self) -> RQResult<()> {
        self.client
            .group_mute(self.code, self.uin, Duration::ZERO)
            .await
    }

    /// 设置管理员。
    pub async fn set_admin(&self) -> RQResult<()> {
        self.client.group_set_admin(self.code, self.uin, true).await
    }

    /// 取消管理员。
    pub async fn unset_admin(&self) -> RQResult<()> {
        self.client
            .group_set_admin(self.code, self.uin, false)
            .await
    }

    /// 踢出群聊。
    pub async fn kick(self, message: impl AsRef<str>, ban: bool) -> RQResult<()> {
        self.client
            .group_kick(self.code, vec![self.uin], message.as_ref(), ban)
            .await
    }

    /// 设置群头衔。
    pub async fn set_special_title(&self, title: impl Into<String>) -> RQResult<()> {
        self.client
            .group_edit_special_title(self.code, self.uin, title.into())
            .await
    }

    /// 设置群名片。
    pub async fn set_card(&self, card: impl Into<String>) -> RQResult<()> {
        self.client
            .edit_group_member_card(self.code, self.uin, card.into())
            .await
    }
}
