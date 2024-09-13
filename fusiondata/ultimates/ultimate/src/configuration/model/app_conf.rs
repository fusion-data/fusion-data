use serde::{Deserialize, Serialize};

use crate::RunMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConf {
  run_mode: RunMode,
  name: String,
}

impl AppConf {
  pub fn run_mode(&self) -> &RunMode {
    &self.run_mode
  }

  pub fn name(&self) -> &str {
    &self.name
  }
}
