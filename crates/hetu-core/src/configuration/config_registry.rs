use std::sync::{Arc, RwLock};

use config::Config;
use serde::de::DeserializeOwned;

use crate::configuration::load_config_with;

use super::{Configurable, ConfigureError, ConfigureResult, HetuSetting, util::load_config};

#[derive(Clone)]
pub struct HetuConfigRegistry {
  config: Arc<RwLock<Arc<Config>>>,
  hetu_setting: Arc<RwLock<Arc<HetuSetting>>>,
}

impl HetuConfigRegistry {
  pub fn builder() -> HetuConfigRegistryBuilder {
    HetuConfigRegistryBuilder::default()
  }

  pub fn new(underling: Arc<Config>, hetu_setting: Arc<HetuSetting>) -> Self {
    Self { config: Arc::new(RwLock::new(underling)), hetu_setting: Arc::new(RwLock::new(hetu_setting)) }
  }

  pub fn reload(&self) -> Result<(), ConfigureError> {
    let config = Arc::new(load_config()?);
    let hetu_setting = Arc::new(HetuSetting::try_from(config.as_ref())?);

    {
      let mut config_write = self.config.write().unwrap();
      *config_write = config.clone();
    }

    let mut hetu_config_write = self.hetu_setting.write().unwrap();
    *hetu_config_write = hetu_setting;

    Ok(())
  }

  pub fn hetu_setting(&self) -> Arc<HetuSetting> {
    self.hetu_setting.read().unwrap().clone()
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

    // 同时更新 hetu_setting
    let new_hetu_config = Arc::new(HetuSetting::try_from(new_config.as_ref())?);

    *config = new_config;
    drop(config); // 释放 config 的写锁

    let mut hetu_config_write = self.hetu_setting.write().unwrap();
    *hetu_config_write = new_hetu_config;

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

impl Default for HetuConfigRegistry {
  fn default() -> Self {
    match Self::builder().build() {
      Ok(c) => c,
      Err(e) => panic!("Error loading configuration: {:?}", e),
    }
  }
}

#[derive(Default)]
pub struct HetuConfigRegistryBuilder {
  config: Option<Config>,
  hetu_setting: Option<HetuSetting>,
}

impl HetuConfigRegistryBuilder {
  pub fn with_config(mut self, config: Config) -> Self {
    self.config = Some(config);
    self
  }

  pub fn with_hetu_config(mut self, hetu_setting: HetuSetting) -> Self {
    self.hetu_setting = Some(hetu_setting);
    self
  }

  pub fn build(self) -> ConfigureResult<HetuConfigRegistry> {
    let config = load_config_with(self.config)?;
    let hetu_setting = match self.hetu_setting {
      Some(hetu_setting) => hetu_setting,
      None => HetuSetting::try_from(&config)?,
    };
    Ok(HetuConfigRegistry::new(Arc::new(config), Arc::new(hetu_setting)))
  }
}
