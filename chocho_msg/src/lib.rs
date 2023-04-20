//! # chocho_msg
//!
//! [chocho](https://github.com/Wybxc/chocho) 的消息处理模块。
//!
//! `chocho_msg` 中的主要数据结构是 [`Message`]，表示一条 QQ 消息。其中包含了消息元素、回复消息、匿名消息等信息。
//!
//! [`Message`] 是对 [`ricq::msg::MessageChain`] 的重新实现，更加方便使用。
//!
//! `chocho_msg` 还添加了用于快速创建 [`Message`] 的宏 [`msg!`]。
//!
//! ## Examples
//!
//! ```
//! use chocho_msg::msg;
//! use chocho_msg::elem::*;
//!
//! let msg = msg!["你好", At::new(12345678)];
//! assert_eq!(msg.to_string(), "你好[@12345678]");
//! ```
#![deny(missing_docs)]
#![feature(let_chains)]

use std::fmt::Display;

use ricq::msg::{
    elem::{Anonymous, Reply},
    MessageElem as OriginMessageElement, PushElem,
};

mod macros;

pub use ricq::msg::elem::RQElem;

/// 消息元素。
///
/// [`ricq::msg::elem`] 的重新导出。
pub mod elem {
    pub use ricq::msg::elem::*;
}

/// 消息。
///
/// `Message` 包含消息元素、回复消息、匿名消息、消息元信息，
/// 是对 [`ricq::msg::MessageChain`] 的重新实现。
#[derive(Debug, Clone, Default)]
pub struct Message {
    /// 回复消息。
    pub reply: Option<Box<Reply>>,
    /// 原始消息元素。
    pub orig_elems: Vec<OriginMessageElement>,
    /// 匿名消息。
    pub anonymous: Option<Box<Anonymous>>,
    /// 消息元信息。
    pub meta: Vec<OriginMessageElement>,
}

impl Message {
    /// 创建一个空的消息。
    pub fn new() -> Self {
        Default::default()
    }

    fn new_with_elems(elems: Vec<OriginMessageElement>) -> Self {
        Self {
            orig_elems: elems,
            ..Default::default()
        }
    }

    /// 遍历消息元素。
    ///
    /// 此方法会返回消息元素副本上的迭代器。
    ///
    /// # Examples
    ///
    /// ```
    /// use chocho_msg::msg;
    ///
    /// let msg = msg!["你好", "世界"];
    /// for elem in msg.elems() {
    ///     assert!(matches!(elem, chocho_msg::RQElem::Text(_)));
    /// }
    /// let chain = ricq::msg::MessageChain::from(msg);
    /// ```
    pub fn elems(&self) -> impl Iterator<Item = RQElem> + '_ {
        self.orig_elems.iter().cloned().map(RQElem::from)
    }

    /// 遍历消息元素。
    ///
    /// 此方法会清空内部的消息元素，并以迭代器的形式返回所有消息元素。
    ///
    /// # Examples
    ///
    /// ```
    /// use chocho_msg::msg;
    ///
    /// let mut msg = msg!["你好", "世界"];
    /// for elem in msg.drain_elems() {
    ///     assert!(matches!(elem, chocho_msg::RQElem::Text(_)));
    /// }
    /// assert!(msg.orig_elems.is_empty());
    /// ```
    pub fn drain_elems(&mut self) -> impl Iterator<Item = RQElem> + '_ {
        self.orig_elems.drain(..).map(RQElem::from)
    }

    /// 遍历消息元素。
    ///
    /// 此方法消耗消息，并以迭代器的形式返回所有消息元素。
    ///
    /// # Examples
    ///
    /// ```
    /// use chocho_msg::msg;
    ///
    /// let msg = msg!["你好", "世界"];
    /// for elem in msg.into_elems() {
    ///     assert!(matches!(elem, chocho_msg::RQElem::Text(_)));
    /// }
    /// ```
    pub fn into_elems(self) -> impl Iterator<Item = RQElem> {
        self.orig_elems.into_iter().map(RQElem::from)
    }

    /// 在消息末尾添加一个消息元素。
    ///
    /// 如果添加的元素与末尾的消息元素都是文本，则会将两个文本合并为一个文本。
    ///
    /// # Examples
    ///
    /// ```
    /// use chocho_msg::msg;
    ///
    /// let mut msg = msg!["你好"];
    /// msg.push("世界");
    /// assert_eq!(msg.to_string(), "你好世界");
    /// ```
    pub fn push(&mut self, elem: impl Into<RQElem>) -> &mut Self {
        match elem.into() {
            RQElem::Text(text) => {
                if let Some(OriginMessageElement::Text(last_text)) = self.orig_elems.last_mut() && last_text.attr6_buf().is_empty() {
                    if let Some(last_str) = &mut last_text.str {
                        last_str.push_str(&text.content);
                    } else {
                        last_text.str = Some(text.content);
                    }
                } else {
                    PushElem::push_to(text, &mut self.orig_elems);
                }
            }
            RQElem::At(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::Face(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::MarketFace(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::Dice(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::FingerGuessing(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::LightApp(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::RichMsg(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::FriendImage(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::GroupImage(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::FlashImage(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::VideoFile(e) => PushElem::push_to(e, &mut self.orig_elems),
            RQElem::Other(e) => self.meta.push(*e),
        }
        self
    }
}

impl From<Message> for ricq::msg::MessageChain {
    fn from(msg: Message) -> Self {
        let mut elems = vec![];
        if let Some(reply) = msg.reply {
            elems.push(OriginMessageElement::from(*reply));
        }
        elems.extend(msg.orig_elems);
        if let Some(anonymous) = msg.anonymous {
            elems.push(OriginMessageElement::from(*anonymous));
        }
        elems.extend(msg.meta);
        ricq::msg::MessageChain::new(elems)
    }
}

impl From<ricq::msg::MessageChain> for Message {
    fn from(msg: ricq::msg::MessageChain) -> Self {
        use ricq::msg::MessageElem as E;
        let mut result = Self::new();
        for elem in msg.0 {
            match elem {
                E::AnonGroupMsg(anon) => {
                    result.anonymous = Some(Box::new(Anonymous::from(anon)));
                }
                E::SrcMsg(src) => {
                    result.reply = Some(Box::new(Reply::from(src)));
                }
                E::Text(_)
                | E::Face(_)
                | E::CommonElem(_)
                | E::MarketFace(_)
                | E::LightApp(_)
                | E::RichMsg(_)
                | E::VideoFile(_)
                | E::NotOnlineImage(_)
                | E::CustomFace(_) => {
                    result.orig_elems.push(elem);
                }
                _ => {
                    result.meta.push(elem);
                }
            }
        }
        result
    }
}

impl<E> FromIterator<E> for Message
where
    RQElem: From<E>,
{
    fn from_iter<T: IntoIterator<Item = E>>(iter: T) -> Self {
        let mut result = Self::new();
        for elem in iter {
            result.push(elem);
        }
        result
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        Self::new_with_elems(vec![OriginMessageElement::Text(ricq_core::pb::msg::Text {
            str: Some(s),
            ..Default::default()
        })])
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for elem in self.elems() {
            elem.fmt(f)?;
        }
        Ok(())
    }
}
