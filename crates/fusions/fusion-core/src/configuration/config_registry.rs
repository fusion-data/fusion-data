use std::sync::{Arc, RwLock};

use config::Config;
use serde::de::DeserializeOwned;

use crate::configuration::load_config_with;

use super::{Configurable, ConfigureError, ConfigureResult, FusionSetting, util::load_config};

#[derive(Clone)]
pub struct FusionConfigRegistry {
  config: Arc<RwLock<Arc<Config>>>,
  fusion_setting: Arc<RwLock<Arc<FusionSetting>>>,
}

impl FusionConfigRegistry {
  pub fn builder() -> FusionConfigRegistryBuilder {
    FusionConfigRegistryBuilder::default()
  }

  pub fn new(underling: Arc<Config>, fusion_setting: Arc<FusionSetting>) -> Self {
    Self { config: Arc::new(RwLock::new(underling)), fusion_setting: Arc::new(RwLock::new(fusion_setting)) }
  }

  pub fn reload(&self) -> Result<(), ConfigureError> {
    let config = Arc::new(load_config()?);
    let fusion_setting = Arc::new(FusionSetting::try_from(config.as_ref())?);

    {
      let mut config_write = self.config.write().unwrap();
      *config_write = config.clone();
    }

    let mut fusion_config_write = self.fusion_setting.write().unwrap();
    *fusion_config_write = fusion_setting;

    Ok(())
  }

  pub fn fusion_setting(&self) -> Arc<FusionSetting> {
    self.fusion_setting.read().unwrap().clone()
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

    // 同时更新 fusion_setting
    let new_fusion_config = Arc::new(FusionSetting::try_from(new_config.as_ref())?);

    *config = new_config;
    drop(config); // 释放 config 的写锁

    let mut fusion_config_write = self.fusion_setting.write().unwrap();
    *fusion_config_write = new_fusion_config;

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

impl Default for FusionConfigRegistry {
  fn default() -> Self {
    match Self::builder().build() {
      Ok(c) => c,
      Err(e) => panic!("Error loading configuration: {:?}", e),
    }
  }
}

#[derive(Default)]
pub struct FusionConfigRegistryBuilder {
  config: Option<Config>,
  fusion_setting: Option<FusionSetting>,
}

impl FusionConfigRegistryBuilder {
  pub fn with_config(mut self, config: Config) -> Self {
    self.config = Some(config);
    self
  }

  pub fn with_fusion_config(mut self, fusion_setting: FusionSetting) -> Self {
    self.fusion_setting = Some(fusion_setting);
    self
  }

  pub fn build(self) -> ConfigureResult<FusionConfigRegistry> {
    let config = load_config_with(self.config)?;
    let fusion_setting = match self.fusion_setting {
      Some(fusion_setting) => fusion_setting,
      None => FusionSetting::try_from(&config)?,
    };
    Ok(FusionConfigRegistry::new(Arc::new(config), Arc::new(fusion_setting)))
  }
}
