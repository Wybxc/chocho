/// 快速创建消息。
///
/// # Examples
///
/// ```
/// use chocho_msg::msg;
/// use chocho_msg::elem::*;
///
/// let message = msg![
///     At::new(12345678),
///     "Hello, world!",
///     Face::new_from_name("笑哭").unwrap(),
/// ];
/// assert_eq!(message.to_string(), "[@12345678]Hello, world![笑哭]");
/// ```
#[macro_export]
macro_rules! msg {
    ($($elem: expr),* $(,)?) => {
        $crate::Message::from_iter([
            $(
                $crate::RQElem::from($elem),
            )*
        ])
    };
}
