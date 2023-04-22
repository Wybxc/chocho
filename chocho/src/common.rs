//! 通用定义导出。

/// 链接分享。
///
/// # Examples
///
/// ```
/// let link = chocho::common::LinkShare {
///     title: "chocho".to_string(),
///     summary: Some("QQ 机器人快速开发框架".to_string()),
///     url: "https://github.com/Wybxc/chocho".to_string(),
///     ..Default::default()
/// };
/// ```
pub use ricq::structs::LinkShare;

/// 音乐分享。
///
/// # Examples
///
/// ```
/// # use chocho::prelude::*;
/// # async fn test(client: RQClient) -> anyhow::Result<()> {
/// let music = chocho::common::MusicShare {
///     title: "アイドル".to_string(),
///     summary: "YOASOBI".to_string(),
///     brief: "[分享]アイドル".to_string(),
///     url: "https://y.music.163.com/m/song?id=2034742057&uct2=LUVOH7rU81LjQU2iHh1WBw%3D%3D&dlt=0846&app_version=8.9.61".to_string(),
///     picture_url: "http://p3.music.126.net/4EQzPt4OaZraiSCRGpINwQ==/109951168506561762.jpg?imageView=1&thumbnail=1440z3063&type=webp&quality=80".to_string(),
///     music_url: "http://music.163.com/song/media/outer/url?id=2034742057&sc=wmv&tn=".to_string(),
/// };
/// let version = chocho::common::MusicVersion::NETEASE;
///
/// client.friend(12345678).share_music(music, version).await?;
/// # Ok(())
/// # }
/// ```
pub use ricq::structs::MusicShare;

/// 音乐分享来源。
///
/// | 值 | 说明 |
/// | --- | --- |
/// | `MusicVersion::QQ` | QQ 音乐 |
/// | `MusicVersion::NETEASE` | 网易云音乐 |
/// | `MusicVersion::MIGU` | 咪咕音乐 |
/// | `MusicVersion::KUGOU` | 酷狗音乐 |
/// | `MusicVersion::KUWO` | 酷我音乐 |
///
/// # Examples
///
/// ```
/// # use chocho::prelude::*;
/// # async fn test(client: RQClient) -> anyhow::Result<()> {
/// let version = chocho::common::MusicVersion::NETEASE;
/// client.friend(12345678).share_music(Default::default(), version).await?;
/// # Ok(())
/// # }
/// ```
pub use ricq::structs::MusicVersion;
