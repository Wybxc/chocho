use std::sync::Arc;

use chocho::ricq::Client;

#[chocho::main]
async fn main(client: Arc<Client>) {
    let account_info = client.account_info.read().await;
    println!("{:?}", account_info);
}
