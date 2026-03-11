use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

use super::{AppSetting, LogSetting, SecuritySetting};

#[derive(Clone, Serialize, Deserialize)]
pub struct UltimateSetting {
  app: AppSetting,

  security: SecuritySetting,

  log: LogSetting,
}

impl UltimateSetting {
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

impl TryFrom<&Config> for UltimateSetting {
  type Error = ConfigError;

  fn try_from(value: &Config) -> core::result::Result<Self, Self::Error> {
    match value.get::<UltimateSetting>("ultimate") {
      Ok(mut setting) => {
        if setting.log.log_name.is_none() || setting.log.log_name.iter().any(|s| s.is_empty()) {
          setting.log.log_name = Some(format!("{}.log", setting.app.name()));
        }
        Ok(setting)
      }
      other => other,
    }
  }
}
