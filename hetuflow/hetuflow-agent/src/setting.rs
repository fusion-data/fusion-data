use serde::{Deserialize, Serialize};
use ultimate_common::ahash::HashMap;

/// Agent 配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentSetting {
  /// Agent 基本信息
  pub agent: AgentConfig,

  /// 连接配置
  pub connection: ConnectionConfig,

  /// 轮询配置
  pub polling: PollingConfig,

  /// 重试配置
  pub retry: RetryConfig,

  /// 进程管理配置
  pub process: ProcessConfig,

  /// 任务执行配置
  pub task: TaskConfig,

  /// 日志配置
  pub logging: LoggingConfig,

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

/// Agent 基本配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentConfig {
  /// Agent 名称
  pub name: String,

  /// Agent 版本
  pub version: String,

  /// Agent 能力列表
  pub capabilities: Vec<String>,

  /// Agent 标签
  pub tags: HashMap<String, String>,

  /// 工作目录
  pub work_dir: String,

  /// 环境变量
  pub env_vars: HashMap<String, String>,
}

/// 连接配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionConfig {
  /// Gateway URL
  pub gateway_url: String,

  /// 连接超时时间（秒）
  pub connect_timeout_seconds: u64,

  /// 心跳间隔（秒）
  pub heartbeat_interval_seconds: u64,

  /// 重连间隔（秒）
  pub reconnect_interval_seconds: u64,

  /// 最大重连次数
  pub max_reconnect_attempts: u32,

  /// 消息缓冲区大小
  pub message_buffer_size: usize,

  /// 启用 TLS
  pub enable_tls: bool,

  /// TLS 证书路径
  pub tls_cert_path: Option<String>,
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

/// 日志配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoggingConfig {
  /// 日志级别
  pub level: String,

  /// 日志格式
  pub format: String,

  /// 日志输出目标
  pub target: LogTarget,

  /// 日志文件路径
  pub file_path: Option<String>,

  /// 日志文件最大大小（字节）
  pub max_file_size: Option<u64>,

  /// 日志文件保留数量
  pub max_files: Option<u32>,

  /// 启用结构化日志
  pub enable_structured_logging: bool,
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

// 默认实现
impl Default for AgentSetting {
  fn default() -> Self {
    Self {
      agent: AgentConfig::default(),
      connection: ConnectionConfig::default(),
      polling: PollingConfig::default(),
      retry: RetryConfig::default(),
      process: ProcessConfig::default(),
      task: TaskConfig::default(),
      logging: LoggingConfig::default(),
      settings: HashMap::default(),
      heartbeat_interval: default_heartbeat_interval(),
      task_timeout: default_task_timeout(),
      max_output_size: default_max_output_size(),
    }
  }
}

impl Default for AgentConfig {
  fn default() -> Self {
    Self {
      name: "hetuflow-agent".to_string(),
      version: "1.0.0".to_string(),
      capabilities: vec!["shell".to_string(), "python".to_string()],
      tags: HashMap::default(),
      work_dir: "/tmp/hetuflow-agent".to_string(),
      env_vars: HashMap::default(),
    }
  }
}

impl Default for ConnectionConfig {
  fn default() -> Self {
    Self {
      gateway_url: "ws://localhost:8080/agent".to_string(),
      connect_timeout_seconds: 30,
      heartbeat_interval_seconds: 30,
      reconnect_interval_seconds: 5,
      max_reconnect_attempts: 10,
      message_buffer_size: 1000,
      enable_tls: false,
      tls_cert_path: None,
    }
  }
}

impl Default for PollingConfig {
  fn default() -> Self {
    Self {
      interval_seconds: 10,
      max_concurrent_tasks: 10,
      capacity_weight: 1.0,
      load_factor_threshold: 0.8,
      enable_adaptive_polling: true,
      min_interval_seconds: 5,
      max_interval_seconds: 60,
    }
  }
}

impl Default for RetryConfig {
  fn default() -> Self {
    Self {
      max_attempts: 3,
      backoff_strategy: BackoffStrategy::Exponential,
      base_delay_ms: 1000,
      max_delay_ms: 30000,
      retry_conditions: vec![RetryCondition::ExitCode(1), RetryCondition::Timeout, RetryCondition::NetworkError],
      enable_jitter: true,
    }
  }
}

impl Default for ProcessConfig {
  fn default() -> Self {
    Self {
      cleanup_interval_seconds: 60,
      zombie_check_interval_seconds: 30,
      process_timeout_seconds: 3600,
      max_concurrent_processes: 50,
      default_resource_limits: ResourceLimits::default(),
      enable_resource_monitoring: true,
      resource_monitor_interval_seconds: 10,
    }
  }
}

impl Default for ResourceLimits {
  fn default() -> Self {
    Self {
      max_memory_bytes: Some(1024 * 1024 * 1024), // 1GB
      max_cpu_percent: Some(80.0),
      max_execution_time_seconds: Some(3600), // 1 hour
      max_file_descriptors: Some(1024),
    }
  }
}

impl Default for TaskConfig {
  fn default() -> Self {
    Self {
      default_timeout_seconds: 3600,
      max_concurrent_tasks: 10,
      output_buffer_size: 8192,
      max_output_size: 10 * 1024 * 1024, // 10MB
      enable_task_isolation: true,
      work_dir_template: "/tmp/hetuflow-task-{task_id}".to_string(),
      cleanup_temp_files: true,
    }
  }
}

impl Default for LoggingConfig {
  fn default() -> Self {
    Self {
      level: "info".to_string(),
      format: "json".to_string(),
      target: LogTarget::Stdout,
      file_path: None,
      max_file_size: Some(100 * 1024 * 1024), // 100MB
      max_files: Some(10),
      enable_structured_logging: true,
    }
  }
}
