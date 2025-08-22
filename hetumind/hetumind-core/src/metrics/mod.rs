#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
  /// 总执行次数
  pub total_executions: u64,
  /// 成功执行次数
  pub successful_executions: u64,
  /// 失败执行次数
  pub failed_executions: u64,
  /// 平均执行时间（毫秒）
  pub avg_execution_time_ms: f64,
  /// P95 执行时间（毫秒）
  pub p95_execution_time_ms: f64,
  /// P99 执行时间（毫秒）
  pub p99_execution_time_ms: f64,
  /// 当前活跃执行数
  pub active_executions: u32,
  /// 队列中等待的任务数
  pub queued_tasks: u32,
  /// 系统资源使用情况
  pub resource_usage: ResourceUsage,
}

impl Default for ExecutionMetrics {
  fn default() -> Self {
    Self {
      total_executions: 0,
      successful_executions: 0,
      failed_executions: 0,
      avg_execution_time_ms: 0.0,
      p95_execution_time_ms: 0.0,
      p99_execution_time_ms: 0.0,
      active_executions: 0,
      queued_tasks: 0,
      resource_usage: ResourceUsage::default(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
  /// CPU 使用率（0.0-1.0）
  pub cpu_usage: f64,
  /// 内存使用量（MB）
  pub memory_usage_mb: u64,
  /// 磁盘使用量（MB）
  pub disk_usage_mb: u64,
  /// 网络 I/O（字节/秒）
  pub network_io_bytes_per_sec: u64,
}

impl Default for ResourceUsage {
  fn default() -> Self {
    Self { cpu_usage: 0.0, memory_usage_mb: 0, disk_usage_mb: 0, network_io_bytes_per_sec: 0 }
  }
}
