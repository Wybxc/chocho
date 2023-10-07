//! 密码登录。

use std::{path::Path, sync::Arc};

use anyhow::{bail, Result};
use futures_util::StreamExt;
use ricq::qsign::QSignClient;
use ricq::{
    handler::Handler, Client, LoginDeviceLocked, LoginNeedCaptcha, LoginResponse, LoginSuccess,
    Protocol,
};
use tokio_util::codec::{FramedRead, LinesCodec};

use crate::login::login_impl;
use crate::AliveHandle;

/// 使用密码登录。
///
/// # Arguments
///
/// * `uin` - QQ 号。
/// * `password` - 密码。
/// * `protocol` - 协议。
/// * `data_folder` - 数据文件夹。
/// * `handler` - 事件处理器。
///
/// # Returns
///
/// 返回一个元组`(Arc<Client>, AliveHandle)`，代表客户端实例和 Keep Alive 的句柄。
///
/// # Examples
///
/// ```no_run
/// use std::{time::Duration, sync::Arc};
/// use chocho_login::{login_with_password, QSignClient};
/// use ricq::handler::DefaultHandler;
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let qsign_client = Arc::new(QSignClient::new(
///         "http://localhost:5000".to_string(),
///         "114514".to_string(),
///         Duration::from_secs(60),
///     )?);
///     let (client, alive) = login_with_password(
///         123456789,
///         "password",
///         ricq::Protocol::AndroidWatch,
///         "./data",
///         qsign_client,
///         DefaultHandler
///     ).await?;
///     alive.auto_reconnect().await?;
/// }
/// ```
pub async fn login_with_password(
    uin: i64,
    password: &str,
    protocol: Protocol,
    data_folder: impl AsRef<Path>,
    qsign_client: Arc<QSignClient>,
    handler: impl Handler + 'static + Send + Sync,
) -> Result<(Arc<Client>, AliveHandle)> {
    login_impl(
        uin,
        protocol,
        data_folder,
        qsign_client,
        handler,
        move |client| async move { password_login(&client, uin, password).await },
    )
    .await
}

/// 密码登录。
///
/// 此方法用于已有客户端实例的情况。
///
/// # Examples
///
/// ```no_run
/// use std::{time::Duration, sync::Arc};
/// use chocho_login::{password::password_login, QSignClient};
/// use ricq::handler::DefaultHandler;
/// use ricq::client::{Connector, DefaultConnector};
/// use ricq::version::get_version;
/// use ricq::ext::common::after_login;
///
/// # async fn _f() -> anyhow::Result<()> {
/// let device = chocho_login::device::random_from_uin(123456789);
/// let protocol = ricq::Protocol::AndroidWatch;
/// let qsign_client = Arc::new(QSignClient::new(
///         "http://localhost:5000".to_string(),
///         "114514".to_string(),
///         Duration::from_secs(60),
///     )?);
/// let client = std::sync::Arc::new(ricq::Client::new(device, get_version(protocol), qsign_client, DefaultHandler));
/// let alive = tokio::spawn({
///     let client = client.clone();
///     let stream = DefaultConnector.connect(&client).await?;
///     async move { client.start(stream).await }
/// });
/// tokio::task::yield_now().await;
/// password_login(&client, 123456789, "password").await?;
/// after_login(&client).await;
/// # Ok(())
/// # }
/// ```
pub async fn password_login(client: &ricq::Client, uin: i64, password: &str) -> Result<()> {
    let resp = client.password_login(uin, password).await?;
    handle_password_login_resp(client, resp).await?;
    Ok(())
}

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
