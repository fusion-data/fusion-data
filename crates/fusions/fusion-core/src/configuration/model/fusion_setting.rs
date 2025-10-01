use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

use crate::configuration::Configurable;

use super::{AppSetting, LogSetting, SecuritySetting};

#[derive(Clone, Serialize, Deserialize)]
pub struct FusionSetting {
  app: AppSetting,

  security: SecuritySetting,

  log: LogSetting,
}

impl Configurable for FusionSetting {
  fn config_prefix() -> &'static str {
    "fusion"
  }
}

impl FusionSetting {
  pub fn app(&self) -> &AppSetting {
    &self.app
  }

  pub fn security(&self) -> &SecuritySetting {
    &self.security
  }

  pub fn log(&self) -> &LogSetting {
    &self.log
  }
}

impl TryFrom<&Config> for FusionSetting {
  type Error = ConfigError;

  fn try_from(value: &Config) -> core::result::Result<Self, Self::Error> {
    value.get(FusionSetting::config_prefix())
  }
}
