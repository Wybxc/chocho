use anyhow::Result;

use chocho::prelude::*;

#[chocho::main]
#[chocho(uin = std::env::var("CHOCHO_NOTIFY_UIN")?.parse()?)]
#[chocho(login_method = chocho::LoginMethod::QrCode)]
async fn main(client: RQClient) -> Result<()> {
    let notify: String = std::env::var("CHOCHO_NOTIFY_PATH")?;
    let target: i64 = std::env::var("CHOCHO_NOTIFY_TARGET")?.parse()?;

    let content = chocho::tokio::fs::read_to_string(&notify).await?;
    client.friend(target).send(content).await?;

    std::process::exit(0);
}
