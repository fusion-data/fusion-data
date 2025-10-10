use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
  /// 重试次数
  pub max_retries: u32,
  /// 重试间隔（秒）
  pub retry_interval_seconds: u64,
}

impl Default for RetryConfig {
  fn default() -> Self {
    Self { max_retries: 3, retry_interval_seconds: 10 }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEngineSetting {
  /// 最大并发执行数
  pub max_concurrent_executions: u32,
  /// 节点执行超时（秒）
  pub node_timeout_seconds: u64,
  /// 工作流执行超时（秒）
  pub workflow_timeout_seconds: u64,
  /// 重试配置
  pub retry_config: RetryConfig,
  /// 内存限制（MB）
  pub memory_limit_mb: u64,
}

impl Default for WorkflowEngineSetting {
  fn default() -> Self {
    Self {
      max_concurrent_executions: 10,
      node_timeout_seconds: 30,
      workflow_timeout_seconds: 30,
      retry_config: RetryConfig::default(),
      memory_limit_mb: 1024,
    }
  }
}
