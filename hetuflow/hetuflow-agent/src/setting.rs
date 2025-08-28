use std::{sync::Arc, time::Duration};

use duration_str::deserialize_duration;
use hetuflow_core::utils::config::write_app_config;
use serde::{Deserialize, Serialize};
use ultimate_common::ahash::HashMap;
use ultimate_core::{DataError, configuration::UltimateConfigRegistry};
use uuid::Uuid;

/// Agent 基本配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentConfig {
  pub agent_id: Uuid,

  /// Agent 名称
  pub name: String,

  /// Agent 版本
  pub version: String,

  /// Agent 能力列表
  pub capabilities: Vec<String>,

  /// Agent 标签
  pub tags: Vec<String>,

  /// 工作目录
  pub work_dir: String,

  /// 环境变量
  pub env_vars: HashMap<String, String>,
}

/// 连接配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionConfig {
  /// Gateway URL
  pub gateway_base_url: String,

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

  /// 消息缓冲区大小
  pub message_buffer_size: usize,

  /// 启用 TLS
  pub enable_tls: bool,

  /// TLS 证书路径
  pub tls_cert_path: Option<String>,
}

impl ConnectionConfig {
  pub fn gateway_url(&self) -> String {
    let path = if self.gateway_base_url.ends_with('/') { "api/v1/gateway/ws" } else { "/api/v1/gateway/ws" };
    format!("{}{}", self.gateway_base_url, path)
  }
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

  /// 最小轮询间隔（秒）
  pub min_interval_seconds: u64,

  /// 最大轮询间隔（秒）
  pub max_interval_seconds: u64,
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
  pub cleanup_interval_seconds: u64,

  /// 僵尸进程检测间隔（秒）
  pub zombie_check_interval_seconds: u64,

  /// 进程超时时间（秒）
  pub process_timeout_seconds: u64,

  /// 最大并发进程数
  pub max_concurrent_processes: usize,

  /// 默认资源限制
  pub default_resource_limits: ResourceLimits,

  /// 启用资源监控
  pub enable_resource_monitoring: bool,

  /// 资源监控间隔（秒）
  pub resource_monitor_interval_seconds: u64,
}

/// 资源限制
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceLimits {
  /// 最大内存使用量（字节）
  pub max_memory_bytes: Option<u64>,

  /// 最大 CPU 使用率（百分比）
  pub max_cpu_percent: Option<f64>,

  /// 最大执行时间（秒）
  pub max_execution_time_seconds: Option<u64>,

  /// 最大文件描述符数
  pub max_file_descriptors: Option<u32>,
}

/// 任务执行配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskConfig {
  /// 默认任务超时时间（秒）
  pub default_timeout_seconds: u64,

  /// 最大并发任务数
  pub max_concurrent_tasks: usize,

  /// 输出缓冲区大小
  pub output_buffer_size: usize,

  /// 最大输出大小（字节）
  pub max_output_size: u64,

  /// 启用任务隔离
  pub enable_task_isolation: bool,

  /// 任务工作目录模板
  pub work_dir_template: String,

  /// 清理临时文件
  pub cleanup_temp_files: bool,
}

/// 日志输出目标
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogTarget {
  /// 标准输出
  Stdout,
  /// 标准错误
  Stderr,
  /// 文件
  File,
  /// 同时输出到控制台和文件
  Both,
}

// 默认值函数
fn default_heartbeat_interval() -> u32 {
  30
}

fn default_task_timeout() -> u32 {
  3600
}

fn default_max_output_size() -> u64 {
  10 * 1024 * 1024 // 10MB
}

/// Agent 配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HetuflowAgentSetting {
  /// Agent 基本信息
  pub agent: Arc<AgentConfig>,

  /// 连接配置
  pub connection: Arc<ConnectionConfig>,

  /// 轮询配置
  pub polling: Arc<PollingConfig>,

  /// 重试配置
  pub retry: Arc<RetryConfig>,

  /// 进程管理配置
  pub process: Arc<ProcessConfig>,

  /// 任务执行配置
  pub task: Arc<TaskConfig>,

  /// 其他配置项（向后兼容）
  pub settings: HashMap<String, String>,

  // 向后兼容的字段
  #[serde(default = "default_heartbeat_interval")]
  pub heartbeat_interval: u32,

  #[serde(default = "default_task_timeout")]
  pub task_timeout: u32,

  #[serde(default = "default_max_output_size")]
  pub max_output_size: u64,
}
const KEY_PATH_AGENT_ID: &str = "hetuflow.agent.agent_id";

impl HetuflowAgentSetting {
  pub fn load(config_registry: &UltimateConfigRegistry) -> Result<Self, DataError> {
    let config = config_registry.config();
    // Check if server_id not exists or invalid uuid in config
    if let Err(_e) = config.get::<Uuid>(KEY_PATH_AGENT_ID) {
      // Generate new UUID and write to config file
      let agent_id = Uuid::new_v4();
      write_app_config("app.toml".into(), KEY_PATH_AGENT_ID, &agent_id.to_string())?;
      // Reload config registry
      config_registry.reload()?;
    }

    let setting = config_registry.get_config_by_path("hetuflow")?;
    Ok(setting)
  }
}
