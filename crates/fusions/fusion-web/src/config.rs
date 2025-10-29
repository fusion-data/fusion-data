use fusion_core::configuration::Configurable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
  pub enable: bool,
  pub server_addr: String,
  pub enable_remote_addr: bool,
}

impl Configurable for WebConfig {
  fn config_prefix() -> &'static str {
    "fusion.web"
  }
}

pub const DEFAULT_CONFIG_STR: &str = include_str!("../resources/default.toml");
