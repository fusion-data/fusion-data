//! 通用配置文件[^config]。
//! 默认配置在 [default.toml] 文件提供。
//!
//! [^config]: 使用了 crate [config](https://docs.rs/config)
mod config_registry;
mod effect;
mod error;
mod model;
mod util;

use serde::de::DeserializeOwned;
pub use ultimate_core_macros::Configuration;

pub use config_registry::*;
pub use effect::*;
pub use error::{ConfigureError, ConfigureResult};
pub use model::*;
pub use util::*;

/// The Configurable trait marks whether the struct can read configuration from the [ConfigRegistry]
pub trait Configurable {
  fn config_prefix() -> &'static str;
}

/// ConfigRegistry is the core trait of configuration management
pub trait ConfigRegistry {
  /// Get the configuration items according to the Configurable's `config_prefix`
  fn get_config<T>(&self) -> ConfigureResult<T>
  where
    T: DeserializeOwned + Configurable;

  fn get_config_by_path<T>(&self, path: &str) -> ConfigureResult<T>
  where
    T: DeserializeOwned;
}
