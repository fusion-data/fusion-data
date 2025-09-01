use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

use crate::configuration::Configurable;

use super::{AppConfig, LogConfig, SecurityConfig};

#[derive(Clone, Serialize, Deserialize)]
pub struct FusionConfig {
  app: AppConfig,

  security: SecurityConfig,

  log: LogConfig,
}

impl Configurable for FusionConfig {
  fn config_prefix() -> &'static str {
    "fusion"
  }
}

impl FusionConfig {
  pub fn app(&self) -> &AppConfig {
    &self.app
  }

  pub fn security(&self) -> &SecurityConfig {
    &self.security
  }

  pub fn log(&self) -> &LogConfig {
    &self.log
  }
}

impl TryFrom<&Config> for FusionConfig {
  type Error = ConfigError;

  fn try_from(value: &Config) -> std::result::Result<Self, Self::Error> {
    value.get(FusionConfig::config_prefix())
  }
}
