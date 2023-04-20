use async_trait::async_trait;
use chocho::prelude::*;
use chocho::ricq::{client::event::FriendMessageEvent, handler::PartlyHandler};

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
            if let Err(e) = client.friend(inner.from_uin).send(response).await {
                tracing::error!("发送消息失败: {}", e);
            }
        }
    }
}

#[chocho::main(handler = Handler)]
async fn main(client: RQClient) {
    let account_info = client.account_info.read().await;
    println!("{:?}", account_info);
}
