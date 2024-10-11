use std::{
  hash::{DefaultHasher, Hash, Hasher},
  time::Duration,
};

use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SchedulerConfigInner {
  advertised_addr: Option<String>,

  #[serde(default = "default_heartbeat_interval")]
  heartbeat_interval: String,

  #[serde(default = "default_alive_timeout")]
  alive_timeout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
  advertised_addr: String,
  heartbeat_interval: Duration,
  alive_timeout: Duration,
  node_id: i64,
}

impl SchedulerConfig {
  pub fn try_new(config: &Config, grpc_sock_addr: String) -> ultimate::Result<Self> {
    let inner: SchedulerConfigInner = config.get("fusion-scheduler")?;
    let advertised_addr = inner.advertised_addr.unwrap_or(grpc_sock_addr);

    let heartbeat_interval = match duration_str::parse_std(inner.heartbeat_interval) {
      Ok(d) => d,
      Err(e) => panic!("Invalid heartbeat_interval: {}", e),
    };

    let alive_timeout = match duration_str::parse_std(inner.alive_timeout) {
      Ok(d) => d,
      Err(e) => panic!("Invalid alive_timeout: {}", e),
    };

    let mut hasher = DefaultHasher::new();
    advertised_addr.hash(&mut hasher);
    let node_id = hasher.finish() as i64;

    Ok(SchedulerConfig { advertised_addr, heartbeat_interval, alive_timeout, node_id })
  }

  pub fn advertised_addr(&self) -> &str {
    &self.advertised_addr
  }

  pub fn heartbeat_interval(&self) -> &Duration {
    &self.heartbeat_interval
  }

  pub fn alive_timeout(&self) -> &Duration {
    &self.alive_timeout
  }

  pub fn node_id(&self) -> i64 {
    self.node_id
  }
}

fn default_heartbeat_interval() -> String {
  "10s".to_string()
}

fn default_alive_timeout() -> String {
  "30s".to_string()
}
