use std::sync::Arc;

use async_trait::async_trait;
use chocho::ricq::{client::event::FriendMessageEvent, handler::PartlyHandler, Client};
use chocho::{Message, RQElem};

struct Handler;

#[async_trait]
impl PartlyHandler for Handler {
    async fn handle_login(&self, uin: i64) {
        tracing::info!("登录成功: {}", uin);
    }
    async fn handle_friend_message(
        &self,
        FriendMessageEvent { client, inner }: FriendMessageEvent,
    ) {
        let message: Message = inner.elements.into();
        let message = message
            .into_elems()
            .filter_map(|elem| match elem {
                RQElem::Text(text) => Some(text.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        if message.trim() == "你好" {
            let response = Message::from("你好".to_string());
            if let Err(e) = client
                .send_friend_message(inner.from_uin, response.into())
                .await
            {
                tracing::error!("发送消息失败: {}", e);
            }
        }
    }
}

#[chocho::main(handler = Handler)]
async fn main(client: Arc<Client>) {
    let account_info = client.account_info.read().await;
    println!("{:?}", account_info);
}
