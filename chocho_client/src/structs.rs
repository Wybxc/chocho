//! 一些结构体。

/// 音频编码。
///
/// # Examples
///
/// ```
/// # use chocho::prelude::*;
/// # async fn test(client: RQClient) -> anyhow::Result<()> {
/// let codec = chocho::common::AudioCodeC::Amr;
/// let group = client.group(12345678);
/// let audio = group.upload_audio("你好".to_string(), codec).await?;
/// group.send_audio(audio).await?;
/// # Ok(())
/// # }
/// ```
pub enum AudioCodeC {
    /// AMR 编码。
    Amr,
    /// SILK 编码。
    Silk,
}
