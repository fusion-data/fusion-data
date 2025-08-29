use std::{path::PathBuf, time::Duration};

use duration_str::deserialize_duration;
use hetuflow_core::utils::config::write_app_config;
use serde::{Deserialize, Serialize};
use ultimate_common::env::get_env;
use ultimate_core::{DataError, configuration::UltimateConfigRegistry};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
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
pub struct HetuflowServerSetting {
  pub max_concurrent_tasks: u32,
  #[serde(deserialize_with = "deserialize_duration")]
  pub history_ttl: Duration,
  pub server: ServerConfig,
}

const KEY_PATH_SERVER_ID: &str = "hetuflow.server.server_id";

impl HetuflowServerSetting {
  pub fn load(config_registry: &UltimateConfigRegistry) -> Result<Self, DataError> {
    let config = config_registry.config();
    // Check if server_id not exists or invalid uuid in config
    if let Err(_e) = config.get::<Uuid>(KEY_PATH_SERVER_ID) {
      // Generate new UUID and write to config file
      let server_id = Uuid::new_v4();
      let path = match get_env("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).join("resources").join("app.toml"),
        Err(_) => PathBuf::from(get_env("HOME")?).join(".hetuflow").join("server.toml"),
      };
      std::fs::create_dir_all(path.parent().unwrap()).unwrap();
      write_app_config(path, KEY_PATH_SERVER_ID, &server_id.to_string())?;
      // Reload config registry
      config_registry.reload()?;
    }

    let setting = config_registry.get_config_by_path("hetuflow")?;
    Ok(setting)
  }
}
