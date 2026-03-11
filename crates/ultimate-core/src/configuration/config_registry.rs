use std::sync::{Arc, RwLock};

use config::Config;
use serde::de::DeserializeOwned;

use crate::configuration::load_config_with;

use super::{Configurable, ConfigureError, ConfigureResult, UltimateSetting, util::load_config};

#[derive(Clone)]
pub struct UltimateConfigRegistry {
  config: Arc<RwLock<Arc<Config>>>,
  ultimate_setting: Arc<RwLock<Arc<UltimateSetting>>>,
}

impl UltimateConfigRegistry {
  pub fn builder() -> UltimateConfigRegistryBuilder {
    UltimateConfigRegistryBuilder::default()
  }

  pub fn new(underling: Arc<Config>, ultimate_setting: Arc<UltimateSetting>) -> Self {
    Self { config: Arc::new(RwLock::new(underling)), ultimate_setting: Arc::new(RwLock::new(ultimate_setting)) }
  }

  pub fn reload(&self) -> Result<(), ConfigureError> {
    let config = Arc::new(load_config()?);
    let ultimate_setting = Arc::new(UltimateSetting::try_from(config.as_ref())?);

    {
      let mut config_write = self.config.write().unwrap();
      *config_write = config.clone();
    }

    let mut ultimate_config_write = self.ultimate_setting.write().unwrap();
    *ultimate_config_write = ultimate_setting;

    Ok(())
  }

  pub fn ultimate_setting(&self) -> Arc<UltimateSetting> {
    self.ultimate_setting.read().unwrap().clone()
  }

  pub fn config(&self) -> Arc<Config> {
    self.config.read().unwrap().clone()
  }

  /// Places the source at the front of the config list only if the key does not already exist,
  /// so that when get_config is called, the source is used for configuration retrieval when the key is missing.
  pub fn prepend_config_source<T>(&self, source: T) -> ConfigureResult<()>
  where
    T: config::Source + Send + Sync + 'static,
  {
    self.add_config_source(source, false)
  }

  /// Appends the source to the end of the config list,
  /// so that when get_config is called, the source overrides existing configuration values.
  pub fn append_config_source<T>(&self, source: T) -> ConfigureResult<()>
  where
    T: config::Source + Send + Sync + 'static,
  {
    self.add_config_source(source, true)
  }

  fn add_config_source<T>(&self, source: T, override_existing: bool) -> ConfigureResult<()>
  where
    T: config::Source + Send + Sync + 'static,
  {
    let mut config = self.config.write().unwrap();
    let c = config.as_ref().clone();
    let b = if override_existing {
      Config::builder().add_source(c).add_source(source)
    } else {
      Config::builder().add_source(source).add_source(c)
    };
    let new_config = Arc::new(b.build()?);

    // 同时更新 ultimate_setting
    let new_ultimate_config = Arc::new(UltimateSetting::try_from(new_config.as_ref())?);

    *config = new_config;
    drop(config); // 释放 config 的写锁

    let mut ultimate_config_write = self.ultimate_setting.write().unwrap();
    *ultimate_config_write = new_ultimate_config;

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
    match Self::builder().build() {
      Ok(c) => c,
      Err(e) => panic!("Error loading configuration: {:?}", e),
    }
  }
}

#[derive(Default)]
pub struct UltimateConfigRegistryBuilder {
  config: Option<Config>,
  ultimate_setting: Option<UltimateSetting>,
}

impl UltimateConfigRegistryBuilder {
  pub fn with_config(mut self, config: Config) -> Self {
    self.config = Some(config);
    self
  }

  pub fn with_ultimate_config(mut self, ultimate_setting: UltimateSetting) -> Self {
    self.ultimate_setting = Some(ultimate_setting);
    self
  }

  pub fn build(self) -> ConfigureResult<UltimateConfigRegistry> {
    let config = load_config_with(self.config)?;
    let ultimate_setting = match self.ultimate_setting {
      Some(ultimate_setting) => ultimate_setting,
      None => UltimateSetting::try_from(&config)?,
    };
    Ok(UltimateConfigRegistry::new(Arc::new(config), Arc::new(ultimate_setting)))
  }
}
