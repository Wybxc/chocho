//! # chocho_login
//!
//! [chocho](https://github.com/Wybxc/chocho) 的登录模块。
//!
//! 该模块提供了密码登录和二维码登录两种方式，支持 iPad、Android 手机、Android 手表、MacOS 客户端和企点协议。
//! 同时提供了自动断线重连功能。
//!
//! ## Examples
//!
//! ```no_run
//! use std::{time::Duration, sync::Arc};
//! use chocho_login::{login, QSignClient};
//! use ricq::handler::DefaultHandler;
//! use anyhow::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let qsign_client = Arc::new(QSignClient::new(
//!         "http://localhost:5000".to_string(),
//!         "114514".to_string(),
//!         Duration::from_secs(60),
//!     )?);
//!     let (client, alive) = login("./data".to_string(), DefaultHandler, None, None, qsign_client).await?;
//!     alive.auto_reconnect().await?;
//! }
//! ```
//! ## Feature flags
//!
//! - `show-qrcode`: 在控制台显示二维码。
//! - `interactive`: 交互式登录。

#![deny(missing_docs)]
#![feature(never_type)]
#![feature(try_blocks)]

use anyhow::Result;
use login::reconnect;
use ricq::{handler::Handler, Client};
use std::{path::PathBuf, sync::Arc};

use tokio::task::JoinHandle;

pub mod device;
mod login;
pub mod password;
pub mod qrcode;

pub use crate::password::login_with_password;
pub use crate::qrcode::login_with_qrcode;
pub use ricq::qsign::QSignClient;

/// 协议。
///
/// | 协议 | 说明 |
/// | --- | --- |
/// | `Protocol::IPad` | iPad 协议 |
/// | `Protocol::AndroidPhone` | Android 手机协议 |
/// | `Protocol::AndroidWatch` | Android 手表协议 |
/// | `Protocol::MacOS` | MacOS 客户端协议 |
/// | `Protocol::QiDian` | 企点协议 |
pub use ricq::Protocol as RQProtocol;

/// 登录保持。
///
/// `AliveHandle` 结构体提供了登录保持的功能，包括等待连接断开、断线重连和自动断线重连。
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn _f(mut alive: chocho_login::AliveHandle) -> anyhow::Result<()> {
    /// alive.alive().await?;
    /// println!("连接已断开");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn alive(&mut self) -> Result<()> {
        if let Some(alive) = self.alive.take() {
            alive.await?;
        }
        Ok(())
    }

    /// 断线重连。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn _f(mut alive: chocho_login::AliveHandle) -> anyhow::Result<()> {
    /// loop {
    ///     alive.alive().await?;
    ///     println!("连接已断开");
    ///     alive.reconnect().await?;
    ///     println!("重连成功");
    /// }
    /// # }
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::{time::Duration, sync::Arc};
    /// use chocho_login::{login, QSignClient};
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
    ///     let (client, alive) = login("./data".to_string(), DefaultHandler, None, None, qsign_client).await?;
    ///     alive.auto_reconnect().await?;
    /// }
    /// ```
    ///
    /// [`alive`]: #method.alive
    /// [`reconnect`]: #method.reconnect
    pub async fn auto_reconnect(mut self) -> Result<!> {
        loop {
            self.alive().await?;
            self.reconnect().await?;
        }
    }
}

/// 登录方式。
pub enum LoginMethod {
    /// 密码登录。
    Password {
        /// 客户端协议。
        protocol: RQProtocol,
        /// 密码。
        password: String,
    },
    /// 二维码登录。
    QrCode,
}

#[cfg(feature = "interactive")]
/// 交互式登录。
///
/// 交互式登录，包含指定账号和登录方式等交互过程。
///
/// # Arguments
///
/// * `data_folder` - 存储数据的目录路径
/// * `handler` - 事件处理器
/// * `uin` - 可选的账号，如果指定则不再交互式询问
/// * `login_method` - 可选的登录方式，如果指定则不再交互式询问
///
/// # Returns
///
/// 包含登录客户端和保持在线句柄的元组。
pub async fn login(
    data_folder: String,
    handler: impl Handler + 'static + Send,
    uin: Option<i64>,
    login_method: Option<LoginMethod>,
    qsign_client: Arc<QSignClient>,
) -> Result<(Arc<Client>, AliveHandle)> {
    use requestty::Question;

    let uin = if let Some(uin) = uin {
        uin
    } else {
        let uin = Question::int("uin").message("请输入账号").build();
        requestty::prompt_one(uin)?.as_int().unwrap()
    };

    let login_method = if let Some(login_method) = login_method {
        login_method
    } else {
        let login_method = Question::select("login_method")
            .message("请选择登录方式：")
            .choice("密码登录")
            .choice("二维码登录")
            .build();
        let login_method = requestty::prompt_one(login_method)?
            .as_list_item()
            .unwrap()
            .index;
        match login_method {
            0 => {
                // 密码登录
                let protocol = Question::select("protocol")
                    .message("请选择客户端协议：")
                    .choice("IPad")
                    .choice("Android Phone")
                    .choice("Android Watch")
                    .choice("MacOS")
                    .choice("企点")
                    .default(0)
                    .build();
                let protocol = requestty::prompt_one(protocol)?
                    .as_list_item()
                    .unwrap()
                    .index;
                let protocol = match protocol {
                    0 => RQProtocol::IPad,
                    1 => RQProtocol::AndroidPhone,
                    2 => RQProtocol::AndroidWatch,
                    3 => RQProtocol::MacOS,
                    4 => RQProtocol::QiDian,
                    _ => unreachable!(),
                };

                let password = Question::password("password")
                    .message("请输入密码")
                    .mask('*')
                    .build();
                let password = requestty::prompt_one(password)?.try_into_string().unwrap();

                LoginMethod::Password { protocol, password }
            }
            1 => {
                // 二维码登录
                LoginMethod::QrCode
            }
            _ => unreachable!(),
        }
    };

    match login_method {
        LoginMethod::Password { protocol, password } => {
            login_with_password(uin, &password, protocol, data_folder, qsign_client, handler).await
        }
        LoginMethod::QrCode => {
            login_with_qrcode(
                uin,
                |img| {
                    println!("{}", qrcode::qrcode_text(&img)?);
                    Ok(())
                },
                data_folder,
                qsign_client,
                handler,
            )
            .await
        }
    }
}
