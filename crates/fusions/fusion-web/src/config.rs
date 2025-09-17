use fusion_core::configuration::Configuration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Configuration)]
#[config_prefix = "fusion.web"]
pub struct WebConfig {
  pub enable: bool,
  pub server_addr: String,
  pub enable_remote_addr: bool,
}

pub const DEFAULT_CONFIG_STR: &str = include_str!("../resources/default.toml");
