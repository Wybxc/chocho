//! # Prelude
//!
//! 此模块导出了 `chocho` 中最常用的类型和特性。
//!
//! ```
//! use chocho::prelude::*;
//! ```

pub use crate::client::{ClientExt, RQClient};
pub use crate::msg::Message;
pub use ricq::msg::elem::RQElem;
pub use ricq::RQResult;
