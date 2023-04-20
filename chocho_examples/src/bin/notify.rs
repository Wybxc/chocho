use std::sync::Arc;

use anyhow::Result;

use chocho::{ricq::Client, Message};

#[chocho::main]
#[chocho(uin = std::env::var("CHOCHO_NOTIFY_UIN")?.parse()?)]
#[chocho(login_method = chocho::LoginMethod::QrCode)]
async fn main(client: Arc<Client>) -> Result<()> {
    let notify: String = std::env::var("CHOCHO_NOTIFY_PATH")?;
    let target: i64 = std::env::var("CHOCHO_NOTIFY_TARGET")?.parse()?;

    let content = chocho::tokio::fs::read_to_string(&notify).await?;
    let message = Message::from(content);
    client.send_friend_message(target, message.into()).await?;

    std::process::exit(0);
}
