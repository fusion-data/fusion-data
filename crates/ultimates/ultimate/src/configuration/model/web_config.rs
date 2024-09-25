use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
