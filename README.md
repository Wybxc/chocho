# chocho

QQ 机器人快速开发框架。

## Examples

```rust
use chocho::prelude::*;
use async_trait::async_trait;
use chocho::ricq::{handler::PartlyHandler};
struct Handler;
#[async_trait]
impl PartlyHandler for Handler {
    async fn handle_login(&self, uin: i64) {
        tracing::info!("登录成功: {}", uin);
    }
}
#[chocho::main(handler = Handler)]
async fn main(client: RQClient) {
    let account_info = client.account_info.read().await;
    tracing::info!("{:?}", account_info);
}
```
