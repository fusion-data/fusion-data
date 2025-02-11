use std::{
  hash::{DefaultHasher, Hash, Hasher},
  time::Duration,
};

use duration_str::deserialize_duration;
use serde::{Deserialize, Serialize};
use ultimate::configuration::Configuration;
use ultimate_common::time::ser::serialize_duration;

#[derive(Debug, Clone, Serialize, Deserialize, Configuration)]
#[config_prefix = "fusion.flow"]
pub struct SchedulerConfig {
  node_id: Option<String>,

  advertised_addr: Option<String>,

  #[serde(
    deserialize_with = "deserialize_duration",
    serialize_with = "serialize_duration",
    default = "default_heartbeat_interval"
  )]
  heartbeat_interval: Duration,

  #[serde(
    deserialize_with = "deserialize_duration",
    serialize_with = "serialize_duration",
    default = "default_alive_timeout"
  )]
  alive_timeout: Duration,
}

impl SchedulerConfig {
  pub fn advertised_addr(&self) -> String {
    self
      .advertised_addr
      .clone()
      .unwrap_or_else(|| std::env::var("ULTIMATE__GRPC__SERVER_ADDR").expect("The gRPC server addr must be set"))
  }

  pub fn node_id(&self) -> String {
    match self.node_id.as_deref() {
      Some(ni) => ni.to_string(),
      None => {
        let advertised_addr = self.advertised_addr();
        let mut hasher = DefaultHasher::new();
        advertised_addr.hash(&mut hasher);
        hasher.finish().to_string()
      }
    }
  }

  pub fn heartbeat_interval(&self) -> &Duration {
    &self.heartbeat_interval
  }

  pub fn alive_timeout(&self) -> &Duration {
    &self.alive_timeout
  }
}

fn default_heartbeat_interval() -> Duration {
  Duration::from_secs(10)
}

fn default_alive_timeout() -> Duration {
  Duration::from_secs(30)
}
