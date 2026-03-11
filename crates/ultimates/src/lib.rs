// fusion
#[cfg(feature = "with-ai")]
pub use ultimate_ai as ai;
pub use ultimate_common as common;
pub use ultimate_core as core;
#[cfg(feature = "with-db")]
pub use ultimate_db as db;
#[cfg(feature = "with-grpc")]
pub use ultimate_grpc as grpc;
#[cfg(feature = "with-security")]
pub use ultimate_security as security;
#[cfg(feature = "with-web")]
pub use ultimate_web as web;

pub use ultimate_core_macros as macros;

#[cfg(feature = "with-web")]
pub mod web_utils;

pub use core::{DataError, Result};
