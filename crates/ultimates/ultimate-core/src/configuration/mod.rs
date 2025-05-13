//! 通用配置文件[^config]。
//! 默认配置在 [default.toml] 文件提供。
//!
//! [^config]: 使用了 crate [config](https://docs.rs/config)
use config::Config;
use serde::de::DeserializeOwned;
use std::{
  env,
  str::FromStr,
  sync::{Arc, RwLock},
};
use ultimate_common::string::b64u_decode;
pub use ultimate_core_macros::Configuration;

mod effect;
mod error;
mod model;
mod util;

pub(crate) use self::util::load_config;
pub use effect::*;
pub use error::{ConfigureError, ConfigureResult};
pub use model::*;

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

#[derive(Clone)]
pub struct UltimateConfigRegistry {
  config: Arc<RwLock<Arc<Config>>>,
  ultimate_config: Arc<UltimateConfig>,
}

impl UltimateConfigRegistry {
  /// ULTIMATE 配置文件根，支持通过环境变量覆盖默认配置。
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use ultimate_core::configuration::*;
  /// # fn test_config_state_from_env() {
  /// // 两个下划线作为层级分隔符
  /// std::env::set_var("ULTIMATE__WEB__SERVER_ADDR", "0.0.0.0:8000");
  ///
  /// std::env::set_var(
  ///     "ULTIMATE__SECURITY__TOKEN__SECRET_KEY",
  ///     "8462b1ec9af827ebed13926f8f1e5409774fa1a21a1c8f726a4a34cf7dcabaf2",
  /// );
  /// std::env::set_var("ULTIMATE__SECURITY__PWD__PWD_KEY", "80c9a35c0f231219ca14c44fe10c728d");
  ///
  /// let configuration = UltimateConfigRegistry::load().unwrap();
  /// let uc = configuration.ultimate_config();
  ///
  /// assert_eq!(uc.security().pwd().pwd_key(), b"80c9a35c0f231219ca14c44fe10c728d");
  /// assert_eq!(
  ///     uc.security().token().secret_key(),
  ///     b"8462b1ec9af827ebed13926f8f1e5409774fa1a21a1c8f726a4a34cf7dcabaf2"
  /// );
  ///
  /// // 由默认配置文件提供
  /// assert_eq!(uc.web().server_addr(), "0.0.0.0:8000");
  /// assert_eq!(uc.app().name(), "ultimate");
  /// # }
  /// ```
  ///
  pub fn load() -> ConfigureResult<Self> {
    let c = load_config()?;
    let ultimate_config = (&c).try_into()?;
    Ok(Self::new(Arc::new(c), Arc::new(ultimate_config)))
  }

  pub(crate) fn new(underling: Arc<Config>, ultimate_config: Arc<UltimateConfig>) -> Self {
    Self { config: Arc::new(RwLock::new(underling)), ultimate_config }
  }

  pub fn ultimate_config(&self) -> &UltimateConfig {
    &self.ultimate_config
  }

  pub fn ultimate_config_arc(&self) -> Arc<UltimateConfig> {
    self.ultimate_config.clone()
  }

  // pub fn config(&self) -> &Config {
  //   self.config.as_ref()
  // }

  pub fn config_arc(&self) -> Arc<Config> {
    self.config.read().unwrap().clone()
  }

  pub fn add_config_source<T>(&self, source: T) -> ConfigureResult<()>
  where
    T: config::Source + Send + Sync + 'static,
  {
    let mut config = self.config.write().unwrap();
    let c = (**config).clone();
    let b = Config::builder().add_source(source).add_source(c);
    *config = Arc::new(b.build()?);
    Ok(())
  }

  pub fn get_config<T>(&self) -> ConfigureResult<T>
  where
    T: DeserializeOwned + Configurable,
  {
    self.get_config_by_path(T::config_prefix())
  }

  pub fn get_config_by_path<T>(&self, path: &str) -> ConfigureResult<T>
  where
    T: DeserializeOwned,
  {
    let c = self.config.read().unwrap().get(path)?;
    Ok(c)
  }
}

impl Default for UltimateConfigRegistry {
  fn default() -> Self {
    match Self::load() {
      Ok(c) => c,
      Err(e) => panic!("Error loading configuration: {:?}", e),
    }
  }
}

pub fn get_env(name: &'static str) -> ConfigureResult<String> {
  env::var(name).map_err(|_| ConfigureError::ConfigMissingEnv(name))
}

pub fn get_env_parse<T: FromStr>(name: &'static str) -> ConfigureResult<T> {
  let val = get_env(name)?;
  val.parse::<T>().map_err(|_| ConfigureError::ConfigWrongFormat(name))
}

pub fn get_env_b64u_as_u8s(name: &'static str) -> ConfigureResult<Vec<u8>> {
  b64u_decode(&get_env(name)?).map_err(|_| ConfigureError::ConfigWrongFormat(name))
}
