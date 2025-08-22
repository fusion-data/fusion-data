use std::sync::{Arc, RwLock};

use config::Config;
use serde::de::DeserializeOwned;

use super::{Configurable, ConfigureError, ConfigureResult, UltimateConfig, util::load_config};

#[derive(Clone)]
pub struct UltimateConfigRegistry {
  config: Arc<RwLock<Arc<Config>>>,
  ultimate_config: Arc<RwLock<Arc<UltimateConfig>>>,
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
  /// ultimate_common::env::set_env("ULTIMATE__WEB__SERVER_ADDR", "0.0.0.0:8000").unwrap();
  ///
  /// ultimate_common::env::set_env(
  ///     "ULTIMATE__SECURITY__TOKEN__SECRET_KEY",
  ///     "8462b1ec9af827ebed13926f8f1e5409774fa1a21a1c8f726a4a34cf7dcabaf2",
  /// ).unwrap();
  /// ultimate_common::env::set_env("ULTIMATE__SECURITY__PWD__PWD_KEY", "80c9a35c0f231219ca14c44fe10c728d").unwrap();
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

  pub fn reload(&self) -> Result<(), ConfigureError> {
    let config = Arc::new(load_config()?);
    let ultimate_config = Arc::new(UltimateConfig::try_from(config.as_ref())?);

    {
      let mut config_write = self.config.write().unwrap();
      *config_write = config.clone();
    }

    let mut ultimate_config_write = self.ultimate_config.write().unwrap();
    *ultimate_config_write = ultimate_config;

    Ok(())
  }

  pub(crate) fn new(underling: Arc<Config>, ultimate_config: Arc<UltimateConfig>) -> Self {
    Self { config: Arc::new(RwLock::new(underling)), ultimate_config: Arc::new(RwLock::new(ultimate_config)) }
  }

  pub fn ultimate_config(&self) -> Arc<UltimateConfig> {
    self.ultimate_config.read().unwrap().clone()
  }

  pub fn config(&self) -> Arc<Config> {
    self.config.read().unwrap().clone()
  }

  pub fn add_config_source<T>(&self, source: T) -> ConfigureResult<()>
  where
    T: config::Source + Send + Sync + 'static,
  {
    let mut config = self.config.write().unwrap();
    let c = config.as_ref().clone();
    let b = Config::builder().add_source(c).add_source(source);
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
