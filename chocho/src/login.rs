//! 登录。
#![allow(clippy::redundant_async_block)]

use std::future::Future;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Result};

use bytes::Bytes;
use futures_util::StreamExt;
use ricq::handler::Handler;
use ricq::{
    client::{Client, Connector, DefaultConnector, NetworkStatus, Token},
    ext::{common::after_login, reconnect::fast_login},
    version::get_version,
    Device, LoginDeviceLocked, LoginNeedCaptcha, LoginResponse, LoginSuccess,
};
use tokio::task::JoinHandle;
use tokio_util::codec::{FramedRead, LinesCodec};

use crate::utils::retry;

/// 协议。
///
/// | 协议 | 说明 |
/// | --- | --- |
/// | [`Protocol::IPad`] | iPad 协议 |
/// | [`Protocol::AndroidPhone`] | Android 手机协议 |
/// | [`Protocol::AndroidWatch`] | Android 手表协议 |
/// | [`Protocol::MacOS`] | MacOS 客户端协议 |
/// | [`Protocol::QiDian`] | 企点协议 |
///
/// # Python
/// ```python
/// class Protocol(Enum):
///     IPAD = enum.auto()
///     ANDROID_PHONE = enum.auto()
///     ANDROID_WATCH = enum.auto()
///     MAC_OS = enum.auto()
///     QI_DIAN = enum.auto()
/// ```
pub use ricq::Protocol;

/// 登录保持。
pub struct AliveHandle {
    client: Arc<ricq::Client>,
    account_data_folder: PathBuf,
    alive: Option<JoinHandle<()>>,
}

impl AliveHandle {
    pub(crate) fn new(
        client: Arc<ricq::Client>,
        account_data_folder: PathBuf,
        alive: JoinHandle<()>,
    ) -> Self {
        Self {
            client,
            account_data_folder,
            alive: Some(alive),
        }
    }

    /// 等待，直到连接断开。
    /// 重复调用会引发 `RuntimeError`。
    pub async fn alive(&mut self) -> Result<()> {
        if let Some(alive) = self.alive.take() {
            alive.await?;
        }
        Ok(())
    }

    /// 断线重连。
    pub async fn reconnect(&mut self) -> Result<()> {
        if self.alive.is_none() {
            // 断线重连
            let handle = reconnect(&self.client, &self.account_data_folder).await?;
            self.alive = Some(handle);
        }
        Ok(())
    }

    /// 开始自动断线重连。
    ///
    /// 此方法相当于无限循环调用 [`alive`] 和 [`reconnect`] 方法。
    pub async fn auto_reconnect(mut self) -> Result<()> {
        loop {
            self.alive().await?;
            self.reconnect().await?;
        }
    }
}

async fn login_impl<Fut>(
    uin: i64,
    protocol: Protocol,
    data_folder: impl AsRef<Path>,
    handler: impl Handler + 'static + Send + Sync,
    login_with_credential: impl FnOnce(Arc<ricq::Client>) -> Fut,
) -> Result<(Arc<Client>, AliveHandle)>
where
    Fut: Future<Output = Result<()>>,
{
    // 创建数据文件夹
    let account_data_folder = data_folder.as_ref().join(uin.to_string());
    tokio::fs::create_dir_all(&account_data_folder).await?;

    let device = load_device_json(uin, &account_data_folder).await?;
    let (client, alive) = prepare_client(device, protocol, handler).await?;

    // 尝试 token 登录
    if !try_token_login(&client, &account_data_folder).await? {
        login_with_credential(client.clone()).await?;
    }

    // 注册客户端，启动心跳。
    after_login(&client).await;
    save_token(&client, &account_data_folder).await?;

    let alive = AliveHandle::new(client.clone(), account_data_folder, alive);
    let client = client;
    Ok((client, alive))
}

/// 使用密码登录。
pub async fn login_with_password(
    uin: i64,
    password: &str,
    protocol: Protocol,
    data_folder: impl AsRef<Path>,
    handler: impl Handler + 'static + Send + Sync,
) -> Result<(Arc<Client>, AliveHandle)> {
    login_impl(
        uin,
        protocol,
        data_folder,
        handler,
        move |client| async move {
            let resp = client.password_login(uin, password).await?;
            handle_password_login_resp(&client, resp).await?;
            Ok(())
        },
    )
    .await
}

/// 使用二维码登录。
pub async fn login_with_qrcode(
    uin: i64,
    show_qrcode: impl FnMut(Bytes) -> Result<()>,
    data_folder: impl AsRef<Path>,
    handler: impl Handler + 'static + Send + Sync,
) -> Result<(Arc<Client>, AliveHandle)> {
    login_impl(
        uin,
        Protocol::AndroidWatch,
        data_folder,
        handler,
        move |client| async move {
            qrcode_login(&client, uin, show_qrcode).await?;
            Ok(())
        },
    )
    .await
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
    handler: impl Handler + 'static + Send + Sync,
) -> tokio::io::Result<(Arc<ricq::Client>, JoinHandle<()>)> {
    let client = Arc::new(ricq::Client::new(device, get_version(protocol), handler));
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

/// 密码登录。
async fn handle_password_login_resp(client: &ricq::Client, mut resp: LoginResponse) -> Result<()> {
    loop {
        match resp {
            LoginResponse::Success(LoginSuccess {
                ref account_info, ..
            }) => {
                tracing::info!("登录成功: {:?}", account_info);
                break;
            }
            LoginResponse::DeviceLocked(LoginDeviceLocked {
                // ref sms_phone,
                verify_url,
                message,
                ..
            }) => {
                bail!(
                    "设备锁：{}\n请前往 {} 解锁",
                    message.unwrap_or_default(),
                    verify_url.unwrap_or_default()
                );
                //也可以走短信验证
                // resp = client.request_sms().await.expect("failed to request sms");
            }
            LoginResponse::NeedCaptcha(LoginNeedCaptcha { ref verify_url, .. }) => {
                tracing::info!("滑块 url: {}", verify_url.as_deref().unwrap_or("")); // TODO: 接入 TxCaptchaHelper
                tracing::info!("请输入 ticket:");
                let mut reader = FramedRead::new(tokio::io::stdin(), LinesCodec::new());
                let ticket = reader.next().await.transpose().unwrap().unwrap();
                resp = client.submit_ticket(&ticket).await?;
            }
            LoginResponse::DeviceLockLogin { .. } => {
                resp = client.device_lock_login().await?;
            }
            LoginResponse::AccountFrozen => bail!("账号被冻结"),
            LoginResponse::TooManySMSRequest => {
                bail!("短信验证码请求过于频繁，请稍后再试")
            }
            unknown => {
                bail!("登录失败: {:?}", unknown)
            }
        }
    }

    Ok(())
}

/// 二维码登录。
pub async fn qrcode_login(
    client: &ricq::Client,
    uin: i64,
    mut show_qrcode: impl FnMut(Bytes) -> Result<()>,
) -> Result<()> {
    use std::time::Duration;

    use ricq::{QRCodeConfirmed, QRCodeImageFetch, QRCodeState};

    tracing::info!("使用二维码登录，uin={}", uin);

    let mut resp = client.fetch_qrcode().await?;

    let mut image_sig = bytes::Bytes::new();
    loop {
        match resp {
            QRCodeState::ImageFetch(QRCodeImageFetch {
                image_data,
                ref sig,
            }) => {
                show_qrcode(image_data)?;
                image_sig = sig.clone();
            }
            QRCodeState::WaitingForScan => {
                tracing::debug!("等待二维码扫描")
            }
            QRCodeState::WaitingForConfirm => {
                tracing::debug!("二维码已扫描，等待确认")
            }
            QRCodeState::Timeout => {
                tracing::info!("二维码已超时，重新获取");
                if let QRCodeState::ImageFetch(QRCodeImageFetch {
                    image_data,
                    ref sig,
                }) = client.fetch_qrcode().await.expect("failed to fetch qrcode")
                {
                    show_qrcode(image_data)?;
                    image_sig = sig.clone();
                }
            }
            QRCodeState::Confirmed(QRCodeConfirmed {
                ref tmp_pwd,
                ref tmp_no_pic_sig,
                ref tgt_qr,
                ..
            }) => {
                tracing::info!("二维码已确认");
                let mut login_resp = client.qrcode_login(tmp_pwd, tmp_no_pic_sig, tgt_qr).await?;
                if let LoginResponse::DeviceLockLogin { .. } = login_resp {
                    login_resp = client.device_lock_login().await?;
                }
                if let LoginResponse::Success(LoginSuccess {
                    ref account_info, ..
                }) = login_resp
                {
                    tracing::info!("登录成功: {:?}", account_info);
                    let real_uin = client.uin().await;
                    if real_uin != uin {
                        tracing::warn!("预期登录账号 {}，但实际登陆账号为 {}", uin, real_uin);
                    }
                    break;
                }
                bail!("登录失败: {:?}", login_resp)
            }
            QRCodeState::Canceled => bail!("二维码已取消"),
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
        resp = client.query_qrcode_result(&image_sig).await?;
    }

    Ok(())
}

/// 断线重连。
pub(crate) async fn reconnect(
    client: &Arc<ricq::Client>,
    account_data_folder: &Path,
) -> Result<JoinHandle<()>> {
    retry(
        10,
        || async {
            // 如果不是网络原因掉线，不重连（服务端强制下线/被踢下线/用户手动停止）
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
            Ok(Ok(alive))
        },
        |e, c| async move {
            tracing::error!("客户端重连失败，原因：{}，剩余尝试 {} 次", e, c);
        },
    )
    .await?
}
