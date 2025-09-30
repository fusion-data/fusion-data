use std::{path::PathBuf, time::Duration};

use duration_str::deserialize_duration;
use fusion_common::env::get_env;
use fusion_core::{DataError, configuration::FusionConfigRegistry};
use hetuflow_core::utils::config::write_app_config;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::JweConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSetting {
  pub server_id: String,
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

  /// Server 过期超时
  #[serde(deserialize_with = "deserialize_duration")]
  pub server_heartbeat_ttl: Duration,

  /// Agent 过期超时
  #[serde(deserialize_with = "deserialize_duration")]
  pub agent_overdue_ttl: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HetuflowSetting {
  pub max_concurrent_tasks: u32,
  #[serde(deserialize_with = "deserialize_duration")]
  pub history_ttl: Duration,
  pub server: ServerSetting,
  /// JWE Token 认证配置
  pub jwe: Option<JweConfig>,
  /// 任务日志配置
  pub task_log: TaskLogConfig,
}

/// 任务日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLogConfig {
  /// 启用任务日志收集
  pub enabled: bool,

  /// 日志存储根目录
  pub log_dir: String,

  /// 日志文件最大大小（字节）
  pub max_file_size: u64,

  /// 日志文件保留天数
  pub retention_days: u32,

  /// 启用日志压缩
  pub enable_compression: bool,

  /// 日志轮转检查间隔（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub rotation_check_interval: Duration,
}

const KEY_PATH_SERVER_ID: &str = "hetuflow.server.server_id";

impl HetuflowSetting {
  pub fn load(config_registry: &FusionConfigRegistry) -> Result<Self, DataError> {
    let config = config_registry.config();
    // Check if server_id not exists or invalid uuid in config
    if let Err(_e) = config.get::<String>(KEY_PATH_SERVER_ID) {
      // Generate new UUID and write to config file
      let server_id = Uuid::new_v4().to_string();
      let path = match get_env("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).join("resources").join("app.toml"),
        Err(_) => PathBuf::from(get_env("HOME")?).join(".hetuflow").join("server.toml"),
      };
      std::fs::create_dir_all(path.parent().unwrap()).unwrap();
      write_app_config(path, KEY_PATH_SERVER_ID, &server_id.to_string()).map_err(|e| DataError::InternalError {
        code: 500,
        msg: format!("Error writing configuration file: {}", e),
        cause: None,
      })?;
      // Reload config registry
      config_registry.reload()?;
    }

    let setting = config_registry.get_config_by_path("hetuflow")?;
    Ok(setting)
  }
}
