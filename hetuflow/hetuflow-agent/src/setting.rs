use std::{env::consts, path::PathBuf, sync::Arc, time::Duration};

use duration_str::deserialize_duration;
use fusion_common::{ahash::HashMap, env::get_env};
use fusion_core::{DataError, configuration::FusionConfigRegistry};
use hetuflow_core::{types::Labels, utils::setting::write_app_setting};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use uuid::Uuid;

/// 连接配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionSetting {
  /// Server URL
  pub server_base_url: String,

  /// 连接超时时间（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub connect_timeout: Duration,

  /// 心跳间隔（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub heartbeat_interval: Duration,

  /// 重连间隔（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub reconnect_interval: Duration,

  /// 最大重连次数
  pub max_reconnect_attempts: u32,
}

/// 轮询配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollingSetting {
  /// 轮询间隔（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub interval: Duration,
}

/// 重试配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RetrySetting {
  /// 最大重试次数
  pub max_attempts: u32,

  /// 退避策略
  pub backoff_strategy: BackoffStrategy,

  /// 基础延迟（毫秒）
  pub base_delay_ms: u64,

  /// 最大延迟（毫秒）
  pub max_delay_ms: u64,

  /// 重试条件
  pub retry_conditions: Vec<RetryCondition>,

  /// 启用抖动
  pub enable_jitter: bool,
}

/// 退避策略
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BackoffStrategy {
  /// 固定延迟
  Fixed,
  /// 线性退避
  Linear,
  /// 指数退避
  Exponential,
  /// 自定义退避
  Custom { multiplier: f64 },
}

/// 重试条件
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RetryCondition {
  /// 退出码
  ExitCode(i32),
  /// 超时
  Timeout,
  /// 进程被杀死
  ProcessKilled,
  /// 资源不足
  ResourceExhausted,
  /// 网络错误
  NetworkError,
  /// 自定义条件
  Custom(String),
}

/// 进程管理配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessSetting {
  /// 进程运行基目录
  run_base_dir: Option<String>,

  /// 清理间隔（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub cleanup_interval: Duration,

  /// 僵尸进程检测间隔（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub zombie_check_interval: Duration,

  /// 进程超时时间（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub process_timeout: Duration,

  /// 最大并发进程数，同时可以调度执行的任务数
  pub max_concurrent_processes: u32,

  /// 启用资源监控
  pub enable_resource_monitoring: bool,

  /// 资源监控间隔（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub resource_monitor_interval: Duration,

  /// 默认资源限制
  pub limits: ResourceLimits,
}

impl ProcessSetting {
  pub fn run_base_dir(&self) -> Result<PathBuf, DataError> {
    match self.run_base_dir.as_deref() {
      Some(dir) => Ok(PathBuf::from(dir)),
      None => {
        let default_dir = dirs::home_dir()
          .ok_or_else(|| DataError::server_error("Failed to get home dir"))?
          .join(".hetu")
          .join("agent")
          .join("runs");
        info!("Default run base dir: {:?}", default_dir);
        Ok(default_dir)
      }
    }
  }
}

/// 资源限制
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceLimits {
  /// 最大内存使用量（字节）
  pub max_memory_bytes: Option<u64>,

  /// 最大 CPU 使用率（百分比）
  pub max_cpu_percent: Option<f64>,
}

/// 任务执行配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskSetting {
  /// 默认任务超时时间（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub default_timeout: Duration,

  /// 最大并发任务数
  pub max_concurrent_tasks: usize,
}

/// Agent 配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HetuflowAgentSetting {
  pub agent_id: String,

  /// Agent 名称
  pub name: Option<String>,

  /// Agent 标签
  pub labels: Labels,

  /// 工作目录
  pub work_dir: Option<String>,

  /// 元数据
  pub metadata: HashMap<String, String>,

  /// JWE Token 配置
  pub jwe_token: Option<String>,

  /// 连接配置
  pub connection: Arc<ConnectionSetting>,

  /// 轮询配置
  pub polling: Arc<PollingSetting>,

  /// 进程管理配置
  pub process: Arc<ProcessSetting>,
}

const KEY_PATH_AGENT_ID: &str = "hetuflow.agent.agent_id";

impl HetuflowAgentSetting {
  pub fn load(config_registry: &FusionConfigRegistry) -> Result<Self, DataError> {
    let default_setting = include_str!("default.toml");
    let setting_source = config::File::from_str(default_setting, config::FileFormat::Toml);
    config_registry.add_config_source(setting_source)?;
    let config = config_registry.config();
    // Check if server_id not exists or invalid uuid in config
    if let Err(e) = config.get::<String>(KEY_PATH_AGENT_ID) {
      warn!("Invalid agent_id in config file, error: {:?}", e);

      // Generate new UUID and write to config file
      let agent_id = Uuid::new_v4().to_string();
      let path = match get_env("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).join("resources").join("app.toml"),
        Err(_) => PathBuf::from(get_env("HOME")?).join(".hetuflow").join("agent.toml"),
      };
      std::fs::create_dir_all(path.parent().unwrap()).unwrap();
      write_app_setting(path, KEY_PATH_AGENT_ID, &agent_id.to_string())
        .map_err(|e| DataError::internal(500, "Error writing configuration file", Some(Box::new(e))))?;
      info!("Generated new agent_id: {}, and write to config file", agent_id);

      // Reload config registry
      config_registry.reload()?;
    }

    let mut setting: HetuflowAgentSetting = config_registry.get_config_by_path("hetuflow.agent")?;
    if let Some(os_name) = System::name() {
      setting.labels.append_label("sys_os", &os_name);
    }
    if let Some(os_version) = System::os_version() {
      setting.labels.append_label("sys_os_version", &os_version);
    }
    if let Some(os_hostname) = System::host_name() {
      setting.labels.append_label("sys_hostname", &os_hostname);
    }
    setting.labels.append_labels([("sys_arch", consts::ARCH), ("sys_family", consts::FAMILY)]);
    Ok(setting)
  }

  /// 获取 Server Gateway WebSocket 地址
  pub fn server_gateway_ws(&self) -> String {
    format!("{}/api/v1/gateway/ws?agent_id={}", self.connection.server_base_url, self.agent_id)
  }
}
