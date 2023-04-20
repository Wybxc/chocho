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
        let elems = ev.inner.elements.0;
        println!("Group Message");
        println!("{:?}", elems);
    }
    async fn handle_friend_message(&self, ev: FriendMessageEvent) {
        let elems = ev.inner.elements.0;
        println!("Friend Message");
        println!("{:?}", elems);
    }
}

#[chocho::main(handler = Handler)]
async fn main(_: RQClient) {}
