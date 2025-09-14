use std::{path::PathBuf, sync::Arc, time::Duration};

use duration_str::deserialize_duration;
use fusion_common::{ahash::HashMap, env::get_env};
use fusion_core::{DataError, configuration::FusionConfigRegistry};
use hetuflow_core::utils::config::write_app_config;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 连接配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionConfig {
  /// Server URL
  pub server_address: String,

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
pub struct PollingConfig {
  /// 轮询间隔（秒）
  pub interval_seconds: u64,

  /// 最大并发任务数
  pub max_concurrent_tasks: usize,

  /// 容量计算权重
  pub capacity_weight: f64,

  /// 负载因子阈值
  pub load_factor_threshold: f64,

  /// 启用自适应轮询
  pub enable_adaptive_polling: bool,
}

/// 重试配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RetryConfig {
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
pub struct ProcessConfig {
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

  /// 日志转发配置
  pub log_forwarding: LogForwardingConfig,
}

/// 日志转发配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogForwardingConfig {
  /// 启用日志转发
  pub enabled: bool,

  /// 日志缓冲区大小（字节）
  pub buffer_size: usize,

  /// 批次大小（条数）
  pub batch_size: usize,

  /// 刷新间隔（毫秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub flush_interval: Duration,

  /// 最大重试次数
  pub max_retries: u32,

  /// 重试间隔（毫秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub retry_interval: Duration,

  /// 启用压缩
  pub enable_compression: bool,
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
pub struct TaskConfig {
  /// 默认任务超时时间（秒）
  #[serde(deserialize_with = "deserialize_duration")]
  pub default_timeout: Duration,

  /// 最大并发任务数
  pub max_concurrent_tasks: usize,
}

/// Agent 配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HetuflowAgentSetting {
  pub agent_id: Uuid,

  /// Agent 名称
  pub name: Option<String>,

  /// Agent 标签
  pub tags: Vec<String>,

  /// 工作目录
  pub work_dir: Option<String>,

  /// 元数据
  pub metadata: HashMap<String, String>,

  /// JWE Token 配置
  pub jwe_token: Option<String>,

  /// 连接配置
  pub connection: Arc<ConnectionConfig>,

  /// 轮询配置
  pub polling: Arc<PollingConfig>,

  /// 进程管理配置
  pub process: Arc<ProcessConfig>,
}

const KEY_PATH_AGENT_ID: &str = "hetuflow.agent.agent_id";

impl HetuflowAgentSetting {
  pub fn load(config_registry: &FusionConfigRegistry) -> Result<Self, DataError> {
    let default_setting = include_str!("default.toml");
    config_registry.add_config_source(config::File::from_str(default_setting, config::FileFormat::Toml))?;
    let config = config_registry.config();
    // Check if server_id not exists or invalid uuid in config
    if let Err(e) = config.get::<Uuid>(KEY_PATH_AGENT_ID) {
      warn!("Invalid agent_id in config file, error: {:?}", e);

      // Generate new UUID and write to config file
      let agent_id = Uuid::new_v4();
      let path = match get_env("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).join("resources").join("app.toml"),
        Err(_) => PathBuf::from(get_env("HOME")?).join(".hetuflow").join("agent.toml"),
      };
      std::fs::create_dir_all(path.parent().unwrap()).unwrap();
      write_app_config(path, KEY_PATH_AGENT_ID, &agent_id.to_string())?;
      info!("Generated new agent_id: {}, and write to config file", agent_id);

      // Reload config registry
      config_registry.reload()?;
    }

    let setting = config_registry.get_config_by_path("hetuflow.agent")?;
    Ok(setting)
  }

  /// 获取 Server Gateway WebSocket 地址
  pub fn server_gateway_ws(&self) -> String {
    format!("ws://{}/api/v1/gateway/ws?agent_id={}", self.connection.server_address, self.agent_id)
  }
}

#[cfg(test)]
mod tests {
  use fusion_common::env::set_env;

  use super::*;

  #[test]
  fn test_dir() {
    for (key, value) in std::env::vars() {
      println!("{}: {}", key, value);
    }
    let dir = std::env::temp_dir();
    println!("dir: {}", dir.display());
    // let file = dir.join("app.toml");
    // assert!(file.exists());
  }

  #[test]
  fn test_load() {
    // 检查配置文件是否存在
    set_env("FUSION_CONFIG_FILE", "resources/app.toml").unwrap();

    // 尝试加载配置
    let config_registry = FusionConfigRegistry::load().unwrap();
    println!("{:?}", config_registry.fusion_config().app());

    let setting = HetuflowAgentSetting::load(&config_registry).unwrap();
    assert!(Uuid::try_parse(&setting.agent_id.to_string()).is_ok());
  }
}
