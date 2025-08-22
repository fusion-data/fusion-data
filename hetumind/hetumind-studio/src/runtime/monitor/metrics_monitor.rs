use hetumind_core::metrics::ExecutionMetrics;
use hetumind_core::workflow::{ExecutionData, NodeExecutionError, NodeName};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct MetricsCollector {
  metrics: Arc<RwLock<ExecutionMetrics>>,
}

impl MetricsCollector {
  pub fn new() -> Self {
    Self { metrics: Arc::new(RwLock::new(ExecutionMetrics::default())) }
  }

  pub async fn record_node_execution(
    &self,
    node_name: &NodeName,
    duration: std::time::Duration,
    result: &Result<Vec<ExecutionData>, NodeExecutionError>,
  ) {
    let metrics = self.metrics.write().await;
    // TODO 更新指标...
  }
}
