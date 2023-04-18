use anyhow::bail;
use futures_util::Future;

/// 自动重试直到得到 `Ok(..)`。
pub async fn retry<F, T, D, E>(
    mut max_count: usize,
    mut f: impl FnMut() -> F,
    mut on_retry: impl FnMut(E, usize) -> D,
) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
    D: Future<Output = ()>,
{
    loop {
        match f().await {
            Ok(t) => return Ok(t),
            Err(e) => {
                if max_count == 0 {
                    return Err(e);
                }
                max_count -= 1;
                on_retry(e, max_count).await;
                tokio::task::yield_now().await;
            }
        }
    }
}

/// 将二维码图片转换为文本形式。
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
