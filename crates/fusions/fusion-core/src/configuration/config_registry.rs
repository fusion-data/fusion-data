use std::sync::{Arc, RwLock};

use config::Config;
use serde::de::DeserializeOwned;

use crate::configuration::load_config_with;

use super::{Configurable, ConfigureError, ConfigureResult, FusionConfig, util::load_config};

#[derive(Clone)]
pub struct FusionConfigRegistry {
  config: Arc<RwLock<Arc<Config>>>,
  fusion_config: Arc<RwLock<Arc<FusionConfig>>>,
}

impl FusionConfigRegistry {
  pub fn builder() -> FusionConfigRegistryBuilder {
    FusionConfigRegistryBuilder::default()
  }

  pub fn new(underling: Arc<Config>, fusion_config: Arc<FusionConfig>) -> Self {
    Self { config: Arc::new(RwLock::new(underling)), fusion_config: Arc::new(RwLock::new(fusion_config)) }
  }

  pub fn reload(&self) -> Result<(), ConfigureError> {
    let config = Arc::new(load_config()?);
    let fusion_config = Arc::new(FusionConfig::try_from(config.as_ref())?);

    {
      let mut config_write = self.config.write().unwrap();
      *config_write = config.clone();
    }

    let mut fusion_config_write = self.fusion_config.write().unwrap();
    *fusion_config_write = fusion_config;

    Ok(())
  }

  pub fn fusion_config(&self) -> Arc<FusionConfig> {
    self.fusion_config.read().unwrap().clone()
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
    let new_config = Arc::new(b.build()?);

    // 同时更新 fusion_config
    let new_fusion_config = Arc::new(FusionConfig::try_from(new_config.as_ref())?);

    *config = new_config;
    drop(config); // 释放 config 的写锁

    let mut fusion_config_write = self.fusion_config.write().unwrap();
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
  fusion_config: Option<FusionConfig>,
}

impl FusionConfigRegistryBuilder {
  pub fn with_config(mut self, config: Config) -> Self {
    self.config = Some(config);
    self
  }

  pub fn with_fusion_config(mut self, fusion_config: FusionConfig) -> Self {
    self.fusion_config = Some(fusion_config);
    self
  }

  pub fn build(self) -> ConfigureResult<FusionConfigRegistry> {
    let config = load_config_with(self.config)?;
    let fusion_config = match self.fusion_config {
      Some(fusion_config) => fusion_config,
      None => FusionConfig::try_from(&config)?,
    };
    Ok(FusionConfigRegistry::new(Arc::new(config), Arc::new(fusion_config)))
  }
}
