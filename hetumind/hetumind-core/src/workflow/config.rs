use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
  /// 重试次数
  pub max_retries: u32,
  /// 重试间隔（秒）
  pub retry_interval_seconds: u64,
  /// 基础延迟时间（毫秒）
  pub base_delay_ms: u64,
  /// 最大延迟时间（毫秒）
  pub max_delay_ms: u64,
  /// 退避倍数
  pub backoff_multiplier: f64,
  /// 可重试的错误类型
  pub retryable_errors: Vec<String>,
}

/// 资源管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagementConfig {
  pub cpu_core_based_scaling: bool,
  pub deadlock_detection_enabled: bool,
  pub resource_allocation_strategy: ResourceAllocationStrategy,
}

/// 资源分配策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAllocationStrategy {
  Equal,    // 平均分配
  Weighted, // 权重分配
  Adaptive, // 自适应分配
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
  pub enable_metrics: bool,
  pub enable_tracing: bool,
  pub metrics_sample_rate: f64,
  pub tracing_sample_rate: f64,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
  pub enable_node_result_cache: bool,
  pub cache_ttl_seconds: u64,
  pub max_cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEngineSetting {
  /// 最大并发执行数
  pub max_concurrent_executions: Option<u32>,
  /// 节点执行超时（秒）
  pub node_timeout_seconds: u64,
  /// 工作流执行超时（秒）
  pub workflow_timeout_seconds: u64,
  /// 内存限制（MB）
  pub memory_limit_mb: u64,
  /// 重试配置
  pub retry_config: RetryConfig,
  pub cache_config: Option<CacheConfig>,
  pub resource_management: ResourceManagementConfig,
}

impl WorkflowEngineSetting {
  pub fn max_concurrent_executions(&self) -> usize {
    match self.max_concurrent_executions {
      Some(v) => v as usize,
      None => num_cpus::get(),
    }
  }
}

impl Default for CacheConfig {
  fn default() -> Self {
    Self { enable_node_result_cache: true, cache_ttl_seconds: 3600, max_cache_size: 1000 }
  }
}
