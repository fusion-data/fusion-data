use std::{path::PathBuf, time::Duration};

use config::Config;
use duration_str::deserialize_duration;
use serde::{Deserialize, Serialize};
use ultimate_core::{
  DataError,
  configuration::{Configuration, UltimateConfigRegistry},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Configuration)]
#[config_prefix = "hetuflow.server"]
pub struct ServerSetting {
  pub server_id: Uuid,
  pub server_name: String,
  pub allow_leader_election: bool,

  /// Job 检查间隔
  #[serde(deserialize_with = "deserialize_duration")]
  pub job_check_interval: Duration,
  #[serde(deserialize_with = "deserialize_duration")]
  pub job_check_duration: Duration,

  #[serde(deserialize_with = "deserialize_duration")]
  pub task_poll_interval: Duration,

  #[serde(deserialize_with = "deserialize_duration")]
  pub agent_heartbeat_interval: Duration,

  #[serde(deserialize_with = "deserialize_duration")]
  pub load_balance_interval: Duration,

  /// Agent 过期超时
  #[serde(deserialize_with = "deserialize_duration")]
  pub agent_heartbeat_ttl: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionSchedulerConfig {
  pub max_concurrent_tasks: u32,
  #[serde(deserialize_with = "deserialize_duration")]
  pub history_ttl: Duration,
  pub server: ServerSetting,
}

const KEY_PATH_SERVER_ID: &str = "hetuflow.server.server_id";

impl FusionSchedulerConfig {
  pub fn load(config_registry: &UltimateConfigRegistry) -> Result<Self, DataError> {
    let config = config_registry.config();
    // Check if server_id not exists or invalid uuid in config
    if let Err(_e) = config.get::<Uuid>(KEY_PATH_SERVER_ID) {
      // Generate new UUID and write to config file
      let server_id = Uuid::new_v4();
      write_app_config("app.toml".into(), server_id)?;
      // Reload config registry
      config_registry.reload()?;
    }

    let hetuflow_config = config_registry.get_config_by_path("hetuflow")?;
    Ok(hetuflow_config)
  }
}

fn write_app_config(path: PathBuf, server_id: Uuid) -> Result<(), DataError> {
  let config = Config::builder()
    .add_source(config::File::from(path.clone()))
    .add_source(config::File::from_str(&format!("{}={}", KEY_PATH_SERVER_ID, server_id), config::FileFormat::Toml))
    .build()?;

  // Convert config to serde_json::Value
  let config_data: serde_json::Value = config.try_deserialize()?;

  // Serialize to TOML string
  let config_str = toml::to_string_pretty(&config_data)
    .map_err(|e| DataError::server_error(format!("TOML serialization error: {}", e)))?;

  // Write the TOML string to file
  std::fs::write(&path, config_str)?;

  Ok(())
}
