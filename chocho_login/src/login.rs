//! 登录。

use std::future::Future;
use std::{path::Path, sync::Arc};

use anyhow::{bail, Result};
use ricq::handler::Handler;
use ricq::qsign::QSignClient;
use ricq::Protocol;
use ricq::{
    client::{Client, Connector, DefaultConnector, NetworkStatus, Token},
    ext::{common::after_login, reconnect::fast_login},
    version::get_version,
    Device, LoginResponse, LoginSuccess,
};
use tokio::task::JoinHandle;

use crate::AliveHandle;

pub(crate) async fn login_impl<Fut>(
    uin: i64,
    protocol: Protocol,
    data_folder: impl AsRef<Path>,
    qsign_client: Arc<QSignClient>,
    handler: impl Handler + 'static + Send,
    login_with_credential: impl FnOnce(Arc<ricq::Client>) -> Fut,
) -> Result<(Arc<Client>, AliveHandle)>
where
    Fut: Future<Output = Result<()>>,
{
    // 创建数据文件夹
    let account_data_folder = data_folder.as_ref().join(uin.to_string());
    tokio::fs::create_dir_all(&account_data_folder).await?;

    let device = load_device_json(uin, &account_data_folder).await?;
    let (client, alive) = prepare_client(device, protocol, qsign_client, handler).await?;

    // 尝试 token 登录
    if !try_token_login(&client, &account_data_folder).await? {
        login_with_credential(client.clone()).await?;
    }

    // 注册客户端，启动心跳。
    after_login(&client).await;
    save_token(&client, &account_data_folder).await?;

    let alive = AliveHandle::new(client.clone(), account_data_folder, alive);
    Ok((client, alive))
}

/// 加载 `device.json`。
async fn load_device_json(uin: i64, data_folder: impl AsRef<Path>) -> Result<Device> {
    use crate::device;

    // 获取 `device.json` 的路径
    let device_json = data_folder.as_ref().join("device.json");

    // 解析设备信息
    let device = if device_json.exists() {
        // 尝试读取已有的 `device.json`
        let json = tokio::fs::read_to_string(device_json).await?;
        device::from_json(&json, &device::random_from_uin(uin))?
    } else {
        // 否则，生成一个新的 `device.json` 并保存到文件中
        let device = device::random_from_uin(uin);
        let json = device::to_json(&device)?;
        tokio::fs::write(device_json, json).await?;
        device
    };

    Ok(device)
}

/// 创建客户端，准备登录。
async fn prepare_client(
    device: Device,
    protocol: Protocol,
    qsign_client: Arc<QSignClient>,
    handler: impl Handler + 'static + Send,
) -> tokio::io::Result<(Arc<ricq::Client>, JoinHandle<()>)> {
    let client = Arc::new(ricq::Client::new(
        device,
        get_version(protocol),
        qsign_client,
        handler,
    ));
    let alive = tokio::spawn({
        let client = client.clone();
        // 连接最快的服务器
        let stream = DefaultConnector.connect(&client).await?;
        async move { client.start(stream).await }
    });

    tokio::task::yield_now().await; // 等一下，确保连上了
    Ok((client, alive))
}

/// 尝试使用 token 登录。
async fn try_token_login(
    client: &ricq::Client,
    account_data_folder: impl AsRef<Path>,
) -> Result<bool> {
    let token_path = account_data_folder.as_ref().join("token.json");

    if !token_path.exists() {
        return Ok(false);
    }
    tracing::info!("发现上一次登录的 token，尝试使用 token 登录");
    let token = tokio::fs::read_to_string(&token_path).await?;
    let token: Token = serde_json::from_str(&token)?;
    match client.token_login(token).await {
        Ok(login_resp) => {
            if let LoginResponse::Success(LoginSuccess {
                ref account_info, ..
            }) = login_resp
            {
                tracing::info!("登录成功: {:?}", account_info);
                return Ok(true);
            }
            bail!("登录失败: {:?}", login_resp)
        }
        Err(_) => {
            tracing::info!("token 登录失败，将删除 token");
            tokio::fs::remove_file(token_path).await?;
            Ok(false)
        }
    }
}

/// 保存 Token，用于断线重连。
async fn save_token(client: &ricq::Client, account_data_folder: impl AsRef<Path>) -> Result<()> {
    let token = client.gen_token().await;
    let token = serde_json::to_string(&token)?;
    let token_path = account_data_folder.as_ref().join("token.json");
    tokio::fs::write(token_path, token).await?;
    Ok(())
}

/// 断线重连。
pub(crate) async fn reconnect(
    client: &Arc<ricq::Client>,
    account_data_folder: &Path,
) -> Result<JoinHandle<()>> {
    let mut retry_count = 10;
    loop {
        match try {
            if client.get_status() != (NetworkStatus::NetworkOffline as u8) {
                bail!("客户端因非网络原因下线，不再重连");
            }
            client.stop(NetworkStatus::NetworkOffline);

            tracing::error!("客户端连接中断，将在 10 秒后重连");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let alive = tokio::spawn({
                let client = client.clone();
                // 连接最快的服务器
                let stream = DefaultConnector.connect(&client).await?;
                async move { client.start(stream).await }
            });
            tokio::task::yield_now().await; // 等一下，确保连上了

            // 启动接收后，再发送登录请求，否则报错 NetworkError
            let token_path = account_data_folder.join("token.json");
            if !token_path.exists() {
                bail!("重连失败：无法找到上次登录的 token");
            }
            let token = tokio::fs::read_to_string(token_path).await?;
            let token = match serde_json::from_str(&token) {
                Ok(token) => token,
                Err(err) => {
                    bail!("重连失败：无法解析上次登录的 token: {}", err)
                }
            };
            fast_login(client, &ricq::ext::reconnect::Credential::Token(token))
                .await
                .map_err(|e| {
                    client.stop(NetworkStatus::NetworkOffline);
                    e
                })?;

            after_login(client).await;

            tracing::info!("客户端重连成功");

            alive
        } {
            Ok(t) => return Ok(t),
            Err(e) => {
                if retry_count == 0 {
                    return Err(e);
                }
                retry_count -= 1;
                tracing::error!("客户端重连失败，原因：{}，剩余尝试 {} 次", e, retry_count);
                tokio::task::yield_now().await;
            }
        }
    }
}
