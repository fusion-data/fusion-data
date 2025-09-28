use std::sync::Arc;

use ahash::HashMap;
use hetumind_core::{
  user::UserId,
  workflow::{ExecutionConfig, NodeExecutionError, WorkflowExecutionError},
};
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::runtime::monitor::ResourceMonitor;

pub struct ConcurrencyController {
  /// 全局执行许可
  execution_semaphore: Arc<Semaphore>,
  /// 节点类型限制
  node_type_limits: HashMap<String, Arc<Semaphore>>,
  /// 用户执行限制
  user_limits: HashMap<UserId, Arc<Semaphore>>,
  /// 资源监控
  resource_monitor: Arc<ResourceMonitor>,
}

impl ConcurrencyController {
  pub fn new(config: ExecutionConfig) -> Self {
    Self {
      execution_semaphore: Arc::new(Semaphore::new(config.max_concurrent_executions as usize)),
      node_type_limits: HashMap::default(),
      user_limits: HashMap::default(),
      resource_monitor: Arc::new(ResourceMonitor::new()),
    }
  }

  pub async fn acquire_execution_permit(&'_ self) -> Result<SemaphorePermit<'_>, WorkflowExecutionError> {
    // 检查系统资源
    self.resource_monitor.check_resources().await?;

    // 获取执行许可
    self.execution_semaphore.acquire().await.map_err(|_| WorkflowExecutionError::ResourceExhausted)
  }

  pub async fn acquire_node_permit(
    &self,
    node_type: &str,
    user_id: Option<UserId>,
  ) -> Result<Vec<SemaphorePermit<'_>>, NodeExecutionError> {
    let mut permits = Vec::new();

    // 节点类型限制
    if let Some(semaphore) = self.node_type_limits.get(node_type) {
      permits.push(semaphore.acquire().await.map_err(|_| NodeExecutionError::ResourceExhausted)?);
    }

    // 用户限制
    if let Some(user_id) = user_id
      && let Some(semaphore) = self.user_limits.get(&user_id)
    {
      permits.push(semaphore.acquire().await.map_err(|_| NodeExecutionError::ResourceExhausted)?);
    }

    Ok(permits)
  }
}
