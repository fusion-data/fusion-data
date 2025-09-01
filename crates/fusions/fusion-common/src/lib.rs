//! crate: fusion_common
//! 常用 Rust 工具库。

pub mod digest;
pub mod env;
mod error;
pub mod helper;
pub mod meta;
pub mod model;
pub mod regex;
pub mod runtime;
pub mod serde;
pub mod string;
pub mod time;
#[cfg(feature = "with-uuid")]
pub mod uuid;

pub mod ahash {
  pub use ::ahash::*;
}

pub use error::{Error, Result};
