//! 查看消息解析后的数据结构，调试用。
use async_trait::async_trait;
use chocho::prelude::*;
use chocho::ricq::{
    client::event::{FriendMessageEvent, GroupMessageEvent},
    handler::PartlyHandler,
};

struct Handler;

#[async_trait]
impl PartlyHandler for Handler {
    async fn handle_group_message(&self, ev: GroupMessageEvent) {
        let msg: Message = ev.inner.elements.into();
        println!("Group Message");
        println!("{:?}", msg.into_elems().collect::<Vec<_>>());
    }
    async fn handle_friend_message(&self, ev: FriendMessageEvent) {
        let msg: Message = ev.inner.elements.into();
        println!("Friend Message");
        println!("{:?}", msg.into_elems().collect::<Vec<_>>());
    }
}

#[chocho::main(handler = Handler)]
async fn main(_: RQClient) {}
