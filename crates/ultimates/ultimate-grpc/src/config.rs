use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use ultimate::configuration::Configurable;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GrpcConfig {
  pub enable: bool,

  pub server_addr: String,

  #[serde(default = "default_plaintext")]
  pub plaintext: bool,

  pub clients: HashMap<String, GrpcClientConfig>,
}

impl Configurable for GrpcConfig {
  fn config_prefix() -> &'static str {
    "ultimate.grpc"
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcClientConfig {
  pub addr: String,

  #[serde(default = "default_plaintext")]
  pub plaintext: bool,
}

fn default_plaintext() -> bool {
  true
}

pub const DEFAULT_CONFIG_STR: &str = include_str!("../resources/default.toml");
