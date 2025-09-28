use std::sync::Arc;

use hetumind_core::workflow::WorkflowExecutionError;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ResourceConfig {
  /// 最大内存使用（MB）
  pub max_memory_mb: u64,
  /// 最大 CPU 使用率
  pub max_cpu_usage: f64,
  /// 最大活跃连接数
  pub max_connections: u32,
  /// 检查间隔（秒）
  pub check_interval_seconds: u64,
}

impl Default for ResourceConfig {
  fn default() -> Self {
    // TODO 从配置中获取
    Self { max_memory_mb: 1024, max_cpu_usage: 0.8, max_connections: 100, check_interval_seconds: 10 }
  }
}

pub struct ResourceMonitor {
  /// 内存使用情况
  memory_usage: Arc<RwLock<u64>>,
  /// CPU 使用情况
  cpu_usage: Arc<RwLock<f64>>,
  /// 活跃连接数
  active_connections: Arc<RwLock<u32>>,
  /// 监控配置
  config: ResourceConfig,
}

impl ResourceMonitor {
  pub fn new() -> Self {
    Self {
      memory_usage: Arc::new(RwLock::new(0)),
      cpu_usage: Arc::new(RwLock::new(0.0)),
      active_connections: Arc::new(RwLock::new(0)),
      config: ResourceConfig::default(),
    }
  }

  pub async fn start_monitoring(&self) {
    let memory_usage = Arc::clone(&self.memory_usage);
    let cpu_usage = Arc::clone(&self.cpu_usage);
    let active_connections = Arc::clone(&self.active_connections);
    let config = self.config.clone();

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(config.check_interval_seconds));

      loop {
        interval.tick().await;

        // 更新系统资源使用情况
        if let Ok(memory) = Self::get_memory_usage().await {
          *memory_usage.write().await = memory;
        }

        if let Ok(cpu) = Self::get_cpu_usage().await {
          *cpu_usage.write().await = cpu;
        }

        // 检查资源限制
        let memory_usage = memory_usage.read().await;
        let cpu_usage = cpu_usage.read().await;
        Self::check_resource_limits(&config, &memory_usage, &cpu_usage).await;
      }
    });
  }

  pub async fn check_resources(&self) -> Result<(), WorkflowExecutionError> {
    let memory = *self.memory_usage.read().await;
    let cpu = *self.cpu_usage.read().await;
    let connections = *self.active_connections.read().await;

    if memory > self.config.max_memory_mb {
      return Err(WorkflowExecutionError::ResourceExhausted);
    }

    if cpu > self.config.max_cpu_usage {
      return Err(WorkflowExecutionError::ResourceExhausted);
    }

    if connections > self.config.max_connections {
      return Err(WorkflowExecutionError::ResourceExhausted);
    }

    Ok(())
  }
}

impl Default for ResourceMonitor {
  fn default() -> Self {
    Self::new()
  }
}

impl ResourceMonitor {
  async fn get_memory_usage() -> Result<u64, WorkflowExecutionError> {
    todo!()
  }

  async fn get_cpu_usage() -> Result<f64, WorkflowExecutionError> {
    todo!()
  }

  async fn check_resource_limits(config: &ResourceConfig, memory_usage: &u64, cpu_usage: &f64) {
    todo!()
  }
}
