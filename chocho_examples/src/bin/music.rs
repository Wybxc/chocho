//! 网易云音乐搜索分享示例。
//!
//! 本示例使用 [NeteaseCloudMusicApi](https://www.npmjs.com/package/NeteaseCloudMusicApi)，
//! 直接运行示例前，请安装 Node.js 与 pnpm。
#![feature(try_blocks)]
use chocho::prelude::*;

use anyhow::{anyhow, Result};
use chocho::ricq::{client::event::FriendMessageEvent, handler::PartlyHandler};
use std::sync::atomic::AtomicU16;
use tokio::process::Command;

/// 随机获取一个可用端口。
fn get_available_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

static PORT: AtomicU16 = AtomicU16::new(0);

struct Handler;
#[async_trait::async_trait]
impl PartlyHandler for Handler {
    async fn handle_friend_message(
        &self,
        FriendMessageEvent { client, inner }: FriendMessageEvent,
    ) {
        let port = PORT.load(std::sync::atomic::Ordering::SeqCst);
        if port == 0 {
            return;
        }

        let message: Message = inner.elements.into();
        let message = message.to_string();

        let mut command = message.split_ascii_whitespace();
        if command.next() != Some("/music") {
            return;
        }
        let keyword = command.collect::<Vec<_>>().join(" ");
        tracing::info!("搜索音乐: {}", keyword);

        let music: Result<_> = try {
            let url = format!("http://localhost:{port}/search?keywords={keyword}&limit=1");
            let result: serde_json::Value = reqwest::get(url).await?.json().await?;
            let result: Option<_> =
                try { result.as_object()?.get("result")?.as_object()?.get("songs") };
            let songs = result.ok_or_else(|| anyhow!("解析 JSON 失败"))?;
            let songs = if let Some(songs) = songs
                .and_then(|songs| songs.as_array())
                .filter(|songs| !songs.is_empty())
            {
                songs
            } else {
                Err(anyhow!("未找到音乐"))?
            };

            let result: Option<_> = try {
                let song = songs[0].as_object()?;
                let id = song.get("id")?.as_u64()?;
                let name = song.get("name")?.as_str()?;
                let album_id = song.get("album")?.as_object()?.get("id")?.as_u64()?;
                (id, name, album_id)
            };
            let (id, name, album_id) = result.ok_or_else(|| anyhow!("解析 JSON 失败"))?;

            let url = format!("http://localhost:{port}/album?id={album_id}");
            let result: serde_json::Value = reqwest::get(url).await?.json().await?;
            let result: Option<_> = try {
                result
                    .as_object()?
                    .get("album")?
                    .as_object()?
                    .get("picUrl")?
                    .as_str()?
            };
            let pic_url = result.ok_or_else(|| anyhow!("解析 JSON 失败"))?;

            let url = format!("http://localhost:{port}/song/url?id={id}");
            let result: serde_json::Value = reqwest::get(url).await?.json().await?;
            let result: Option<_> = try {
                result
                    .as_object()?
                    .get("data")?
                    .as_array()?
                    .get(0)?
                    .as_object()?
                    .get("url")?
                    .as_str()?
            };
            let music_url = result.ok_or_else(|| anyhow!("解析 JSON 失败"))?;

            chocho::common::MusicShare {
                title: name.to_string(),
                summary: "来自网易云音乐".to_string(),
                brief: format!("[分享]{name}"),
                url: format!("https://y.music.163.com/m/song?id={id}"),
                picture_url: pic_url.to_string(),
                music_url: music_url.to_string(),
            }
        };
        let result: Result<()> = try {
            match music {
                Ok(music) => {
                    client
                        .friend(inner.from_uin)
                        .share_music(music, chocho::common::MusicVersion::NETEASE)
                        .await?;
                }
                Err(e) => {
                    let message = format!("查询音乐失败: {}", e);
                    client.friend(inner.from_uin).send(message).await?;
                }
            }
        };
        if let Err(e) = result {
            tracing::error!("处理音乐分享失败: {}", e);
        }
    }
}

#[chocho::main(handler = Handler)]
async fn main(_client: RQClient) -> Result<()> {
    let port = get_available_port()?;
    let mut child = Command::new("pnpm")
        .arg("--package=qrcode@1.5.1")
        .arg("--package=NeteaseCloudMusicApi")
        .arg("dlx")
        .arg("NeteaseCloudMusicApi")
        .env("PORT", port.to_string())
        .spawn()?;
    chocho::finalizer(move || async move {
        child.kill().await.unwrap();
    });
    PORT.store(port, std::sync::atomic::Ordering::SeqCst);

    tracing::info!("NeteaseCloudMusicApi started on port {}", port);

    Ok(())
}
