use serde::{Deserialize, Serialize};
use fusion_core::configuration::Configuration;

#[derive(Debug, Clone, Serialize, Deserialize, Configuration)]
#[config_prefix = "fusion.web"]
pub struct WebConfig {
  enable: bool,
  server_addr: String,
}

impl WebConfig {
  pub fn enable(&self) -> bool {
    self.enable
  }

  pub fn server_addr(&self) -> &str {
    &self.server_addr
  }
}

pub const DEFAULT_CONFIG_STR: &str = include_str!("../resources/default.toml");
