use std::sync::Arc;

use async_trait::async_trait;
use chocho::ricq::{
    client::event::FriendMessageEvent, handler::PartlyHandler, msg::MessageChainBuilder, Client,
};

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
        let message = inner
            .elements
            .into_iter()
            .filter_map(|e| {
                if let chocho::ricq::msg::elem::RQElem::Text(t) = e {
                    Some(t.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        let message = message.join("").trim().to_string();

        let mut builder = MessageChainBuilder::new();
        builder.push_str("你好");
        let response = builder.build();

        if message == "你好" {
            if let Err(e) = client.send_friend_message(inner.from_uin, response).await {
                tracing::error!("发送消息失败: {}", e);
            }
        }
    }
}

#[chocho::main]
#[handler = Handler]
async fn main(client: Arc<Client>) {
    let account_info = client.account_info.read().await;
    println!("{:?}", account_info);
}
