pub mod application;
pub mod component;
pub mod configuration;
mod data_error;
pub mod log;
pub mod metas;
mod model;
pub mod plugin;
mod run_mode;
pub mod security;
pub mod signal;
pub mod timer;
pub mod utils;

pub use async_trait::async_trait;
pub use data_error::*;
pub use model::*;
pub use run_mode::*;
pub use ultimate_core_macros::Builder;

pub type Result<T> = core::result::Result<T, DataError>;
