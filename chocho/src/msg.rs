//! 消息处理。
use std::fmt::Display;

use ricq::msg::{
    elem::{Anonymous, RQElem, Reply},
    MessageElem as OriginMessageElement,
};

/// 消息。
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
    pub fn elems(&self) -> impl Iterator<Item = RQElem> + '_ {
        self.orig_elems.iter().cloned().map(RQElem::from)
    }

    /// 遍历消息元素。
    pub fn drain_elems(&mut self) -> impl Iterator<Item = RQElem> + '_ {
        self.orig_elems.drain(..).map(RQElem::from)
    }

    /// 遍历消息元素。
    pub fn into_elems(self) -> impl Iterator<Item = RQElem> {
        self.orig_elems.into_iter().map(RQElem::from)
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
        use ricq::msg::PushElem;

        let mut result = Self::new();
        for elem in iter {
            match RQElem::from(elem) {
                RQElem::At(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::Text(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::Face(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::MarketFace(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::Dice(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::FingerGuessing(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::LightApp(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::RichMsg(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::FriendImage(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::GroupImage(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::FlashImage(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::VideoFile(e) => PushElem::push_to(e, &mut result.orig_elems),
                RQElem::Other(e) => result.meta.push(*e),
            }
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
