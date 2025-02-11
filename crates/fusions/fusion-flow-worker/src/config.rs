use std::time::Duration;

use duration_str::deserialize_duration;
use serde::{Deserialize, Serialize};
use ultimate::configuration::Configuration;
use ultimate_common::time::ser::serialize_duration;

#[derive(Clone, Serialize, Deserialize, Configuration)]
#[config_prefix = "fusion.scheduler.worker"]
pub struct WorkerConfig {
  pub(crate) node_seeds: Vec<String>,

  #[serde(deserialize_with = "deserialize_duration", serialize_with = "serialize_duration")]
  pub(crate) heartbeat_interval: Duration,

  pub(crate) namespaces: Vec<String>,

  pub(crate) token: String,
}
