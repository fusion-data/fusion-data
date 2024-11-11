use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

use crate::configuration::Configurable;

use super::{AppConfig, DbConfig, GrpcConfig, LogConfig, SecurityConfig, WebConfig};

#[derive(Clone, Serialize, Deserialize)]
pub struct UltimateConfig {
  app: AppConfig,

  security: SecurityConfig,

  db: DbConfig,

  log: LogConfig,

  web: WebConfig,

  grpc: GrpcConfig,
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

  pub fn web(&self) -> &WebConfig {
    &self.web
  }

  pub fn security(&self) -> &SecurityConfig {
    &self.security
  }

  pub fn db(&self) -> &DbConfig {
    &self.db
  }

  pub fn log(&self) -> &LogConfig {
    &self.log
  }

  pub fn grpc(&self) -> &GrpcConfig {
    &self.grpc
  }
}

impl TryFrom<&Config> for UltimateConfig {
  type Error = ConfigError;

  fn try_from(value: &Config) -> std::result::Result<Self, Self::Error> {
    value.get(UltimateConfig::config_prefix())
  }
}
