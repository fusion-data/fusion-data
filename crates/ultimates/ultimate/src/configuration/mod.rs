//! 通用配置文件[^config]。
//! 默认配置在 [default.toml] 文件提供。
//!
//! [^config]: 使用了 crate [config](https://docs.rs/config)
use config::Config;
use serde::{Deserialize, Serialize};
use std::{env, str::FromStr, sync::Arc};

use ultimate_common::string::b64u_decode;

mod effect;
mod error;
pub mod model;
mod util;

pub(crate) use self::util::load_config;
pub use effect::*;
pub use error::{Error, Result};
use model::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Configuration {
  app: AppConf,

  security: SecurityConf,

  db: DbConf,

  tracing: TracingConfig,

  web: WebConfig,

  grpc: GrpcConf,
}

impl Configuration {
  pub fn app(&self) -> &AppConf {
    &self.app
  }

  pub fn web(&self) -> &WebConfig {
    &self.web
  }

  pub fn security(&self) -> &SecurityConf {
    &self.security
  }

  pub fn db(&self) -> &DbConf {
    &self.db
  }

  pub fn tracing(&self) -> &TracingConfig {
    &self.tracing
  }

  pub fn grpc(&self) -> &GrpcConf {
    &self.grpc
  }
}

impl TryFrom<&Config> for Configuration {
  type Error = Error;

  fn try_from(c: &Config) -> std::result::Result<Self, Self::Error> {
    let qc = c.get::<Configuration>("ultimate")?;
    Ok(qc)
  }
}

#[derive(Clone)]
pub struct ConfigurationState {
  underling: Arc<Config>,
  configuration: Arc<Configuration>,
}

impl ConfigurationState {
  /// ULTIMATE 配置文件根，支持通过环境变量覆盖默认配置。
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use ultimate::configuration::{ConfigState, model::*};
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
  /// let config_state = ConfigState::load().unwrap();
  /// let qc = config_state.ultimate_config();
  ///
  /// assert_eq!(qc.security().pwd().pwd_key(), b"80c9a35c0f231219ca14c44fe10c728d");
  /// assert_eq!(
  ///     qc.security().token().secret_key(),
  ///     b"8462b1ec9af827ebed13926f8f1e5409774fa1a21a1c8f726a4a34cf7dcabaf2"
  /// );
  ///
  /// // 由默认配置文件提供
  /// assert_eq!(qc.web().server_addr(), "0.0.0.0:8000");
  /// assert_eq!(qc.app().name(), "ultimate");
  /// # }
  /// ```
  ///
  pub fn load() -> Result<Self> {
    let c = load_config()?;
    let ultimate_config = Configuration::try_from(&c)?;
    Ok(Self::new(Arc::new(c), Arc::new(ultimate_config)))
  }

  pub(crate) fn new(underling: Arc<Config>, ultimate_config: Arc<Configuration>) -> Self {
    Self { underling, configuration: ultimate_config }
  }

  pub fn configuration(&self) -> &Configuration {
    self.configuration.as_ref()
  }

  pub fn configuration_clone(&self) -> Arc<Configuration> {
    self.configuration.clone()
  }

  pub fn underling(&self) -> &Config {
    self.underling.as_ref()
  }

  pub fn underling_clone(&self) -> Arc<Config> {
    self.underling.clone()
  }
}

pub fn get_env(name: &'static str) -> Result<String> {
  env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

pub fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
  let val = get_env(name)?;
  val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}

pub fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
  b64u_decode(&get_env(name)?).map_err(|_| Error::ConfigWrongFormat(name))
}
