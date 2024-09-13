use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GrpcConf {
  pub enable: bool,

  pub server_addr: String,

  pub plaintext: bool,

  pub clients: HashMap<String, GrpcClientConf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcClientConf {
  pub addr: String,

  pub plaintext: bool,
}
