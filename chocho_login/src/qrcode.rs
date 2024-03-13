//! 二维码登录。
use std::{path::Path, sync::Arc};

use anyhow::{bail, Result};
use bytes::Bytes;
use ricq::qsign::QSignClient;
use ricq::{handler::Handler, Client, LoginResponse, LoginSuccess, Protocol};

use crate::login::login_impl;
use crate::AliveHandle;

/// 使用二维码登录。
///
/// # Arguments
///
/// * `uin` - QQ号
/// * `show_qrcode` - 可以展示二维码的回调函数
/// * `data_folder` - 数据文件夹
/// * `handler` - 实例化的事件处理器
///
/// # Returns
///
/// 返回一个元组`(Arc<Client>, AliveHandle)`，代表客户端实例和 Keep Alive 的句柄。
///
/// # Examples
///
/// ```no_run
/// use std::{time::Duration, sync::Arc};
/// use chocho_login::{login_with_qrcode, QSignClient};
/// use chocho_login::qrcode::qrcode_text;
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
///     let (client, alive) = login_with_qrcode(123456789, |qrcode| {
///         println!("{}", qrcode_text(&qrcode)?);
///         Ok(())
///     }, "./data", qsign_client, DefaultHandler).await?;
///     alive.auto_reconnect().await?;
/// }
/// ```
pub async fn login_with_qrcode(
    uin: i64,
    show_qrcode: impl FnMut(Bytes) -> Result<()>,
    data_folder: impl AsRef<Path>,
    qsign_client: Arc<QSignClient>,
    handler: impl Handler + 'static + Send,
) -> Result<(Arc<Client>, AliveHandle)> {
    login_impl(
        uin,
        Protocol::AndroidWatch,
        data_folder,
        qsign_client,
        handler,
        move |client| async move { qrcode_login(&client, uin, show_qrcode).await },
    )
    .await
}

/// 二维码登录。
///
/// 此方法用于已有客户端实例的情况。
///
/// # Examples
///
/// ```no_run
/// use std::{time::Duration, sync::Arc};
/// use chocho_login::QSignClient;
/// use chocho_login::qrcode::qrcode_login;
/// use chocho_login::qrcode::qrcode_text;
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
/// qrcode_login(&client, 123456789, |qrcode| {
///     println!("{}", qrcode_text(&qrcode)?);
///     Ok(())
/// }).await?;
/// after_login(&client).await;
/// # Ok(())
/// # }
/// ```
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

/// 将二维码图片转换为文本形式。
#[cfg(feature = "show-qrcode")]
pub fn qrcode_text(qrcode: &[u8]) -> anyhow::Result<String> {
    let qrcode = image::load_from_memory(qrcode)?.to_luma8();
    let mut qrcode = rqrr::PreparedImage::prepare(qrcode);
    let grids = qrcode.detect_grids();
    if grids.len() != 1 {
        bail!("无法识别二维码");
    }
    let (_, content) = grids[0].decode()?;
    let qrcode = qrcode::QrCode::new(content)?;
    let qrcode = qrcode.render::<qrcode::render::unicode::Dense1x2>().build();
    Ok(qrcode)
}
