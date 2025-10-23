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

impl RetryConfig {
  pub fn new(
    max_retries: u32,
    retry_interval_seconds: u64,
    base_delay_ms: u64,
    max_delay_ms: u64,
    backoff_multiplier: f64,
  ) -> Self {
    Self {
      max_retries,
      retry_interval_seconds,
      base_delay_ms,
      max_delay_ms,
      backoff_multiplier,
      retryable_errors: Vec::default(),
    }
  }

  pub fn with_max_retries(mut self, max_retries: u32) -> Self {
    self.max_retries = max_retries;
    self
  }

  pub fn with_retry_interval_seconds(mut self, retry_interval_seconds: u64) -> Self {
    self.retry_interval_seconds = retry_interval_seconds;
    self
  }

  pub fn with_base_delay_ms(mut self, base_delay_ms: u64) -> Self {
    self.base_delay_ms = base_delay_ms;
    self
  }

  pub fn with_max_delay_ms(mut self, max_delay_ms: u64) -> Self {
    self.max_delay_ms = max_delay_ms;
    self
  }

  pub fn with_backoff_multiplier(mut self, backoff_multiplier: f64) -> Self {
    self.backoff_multiplier = backoff_multiplier;
    self
  }

  pub fn with_retryable_errors<I, V>(mut self, retryable_errors: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<String>,
  {
    self.retryable_errors = retryable_errors.into_iter().map(|v| v.into()).collect();
    self
  }

  pub fn add_retryable_error(mut self, retryable_error: impl Into<String>) -> Self {
    self.retryable_errors.push(retryable_error.into());
    self
  }
}

/// 资源管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagementConfig {
  pub cpu_core_based_scaling: bool,
  pub deadlock_detection_enabled: bool,
  pub resource_allocation_strategy: ResourceAllocationStrategy,
}

impl ResourceManagementConfig {
  pub fn new(resource_allocation_strategy: ResourceAllocationStrategy) -> Self {
    Self {
      cpu_core_based_scaling: Default::default(),
      deadlock_detection_enabled: Default::default(),
      resource_allocation_strategy,
    }
  }

  pub fn with_cpu_core_based_scaling(mut self, cpu_core_based_scaling: bool) -> Self {
    self.cpu_core_based_scaling = cpu_core_based_scaling;
    self
  }

  pub fn with_deadlock_detection_enabled(mut self, deadlock_detection_enabled: bool) -> Self {
    self.deadlock_detection_enabled = deadlock_detection_enabled;
    self
  }

  pub fn with_resource_allocation_strategy(mut self, resource_allocation_strategy: ResourceAllocationStrategy) -> Self {
    self.resource_allocation_strategy = resource_allocation_strategy;
    self
  }
}

/// 资源分配策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

impl MonitoringConfig {
  pub fn new(metrics_sample_rate: f64, tracing_sample_rate: f64) -> Self {
    Self {
      enable_metrics: Default::default(),
      enable_tracing: Default::default(),
      metrics_sample_rate,
      tracing_sample_rate,
    }
  }

  pub fn with_enable_metrics(mut self, enable_metrics: bool) -> Self {
    self.enable_metrics = enable_metrics;
    self
  }

  pub fn with_enable_tracing(mut self, enable_tracing: bool) -> Self {
    self.enable_tracing = enable_tracing;
    self
  }

  pub fn with_metrics_sample_rate(mut self, metrics_sample_rate: f64) -> Self {
    self.metrics_sample_rate = metrics_sample_rate;
    self
  }

  pub fn with_tracing_sample_rate(mut self, tracing_sample_rate: f64) -> Self {
    self.tracing_sample_rate = tracing_sample_rate;
    self
  }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
  pub enable_node_result_cache: bool,
  pub cache_ttl_seconds: u64,
  pub max_cache_size: usize,
}

impl CacheConfig {
  pub fn new(cache_ttl_seconds: u64, max_cache_size: usize) -> Self {
    Self { enable_node_result_cache: Default::default(), cache_ttl_seconds, max_cache_size }
  }

  pub fn with_enable_node_result_cache(mut self, enable_node_result_cache: bool) -> Self {
    self.enable_node_result_cache = enable_node_result_cache;
    self
  }

  pub fn with_cache_ttl_seconds(mut self, cache_ttl_seconds: u64) -> Self {
    self.cache_ttl_seconds = cache_ttl_seconds;
    self
  }

  pub fn with_max_cache_size(mut self, max_cache_size: usize) -> Self {
    self.max_cache_size = max_cache_size;
    self
  }
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
  pub fn new(
    node_timeout_seconds: u64,
    workflow_timeout_seconds: u64,
    memory_limit_mb: u64,
    retry_config: RetryConfig,
    resource_management: ResourceManagementConfig,
  ) -> Self {
    Self {
      max_concurrent_executions: Default::default(),
      node_timeout_seconds,
      workflow_timeout_seconds,
      memory_limit_mb,
      retry_config,
      cache_config: Default::default(),
      resource_management,
    }
  }

  pub fn with_max_concurrent_executions(mut self, max_concurrent_executions: u32) -> Self {
    self.max_concurrent_executions = Some(max_concurrent_executions);
    self
  }

  pub fn with_node_timeout_seconds(mut self, node_timeout_seconds: u64) -> Self {
    self.node_timeout_seconds = node_timeout_seconds;
    self
  }

  pub fn with_workflow_timeout_seconds(mut self, workflow_timeout_seconds: u64) -> Self {
    self.workflow_timeout_seconds = workflow_timeout_seconds;
    self
  }

  pub fn with_memory_limit_mb(mut self, memory_limit_mb: u64) -> Self {
    self.memory_limit_mb = memory_limit_mb;
    self
  }

  pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
    self.retry_config = retry_config;
    self
  }

  pub fn with_cache_config(mut self, cache_config: CacheConfig) -> Self {
    self.cache_config = Some(cache_config);
    self
  }

  pub fn with_resource_management(mut self, resource_management: ResourceManagementConfig) -> Self {
    self.resource_management = resource_management;
    self
  }
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
