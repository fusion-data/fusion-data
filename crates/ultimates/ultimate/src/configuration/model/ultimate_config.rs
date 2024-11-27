use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

use crate::configuration::Configurable;

use super::{AppConfig, LogConfig, SecurityConfig};

#[derive(Clone, Serialize, Deserialize)]
pub struct UltimateConfig {
  app: AppConfig,

  security: SecurityConfig,

  log: LogConfig,
}

impl Configurable for UltimateConfig {
  fn config_prefix() -> &'static str {
    "ultimate"
  }
}

impl UltimateConfig {
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

impl TryFrom<&Config> for UltimateConfig {
  type Error = ConfigError;

  fn try_from(value: &Config) -> std::result::Result<Self, Self::Error> {
    value.get(UltimateConfig::config_prefix())
  }
}
