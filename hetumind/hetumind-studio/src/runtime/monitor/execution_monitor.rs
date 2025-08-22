use std::sync::Arc;

use hetumind_core::workflow::{Execution, ExecutionId, NodeExecutionError, NodeName, VecExecutionData};
use log::{debug, error, info};
use metrics::{counter, gauge, histogram};

use super::MetricsCollector;

pub struct ExecutionMonitor {
  /// 执行指标收集器
  metrics_collector: Arc<MetricsCollector>,
  // 日志记录器
  // logger: Arc<Logger>,
  // 追踪器
  // tracer: Arc<Tracer>,
}

impl ExecutionMonitor {
  pub fn new() -> Self {
    Self { metrics_collector: Arc::new(MetricsCollector::new()) }
  }
}

impl ExecutionMonitor {
  pub async fn start_execution_monitoring(&self, execution: &Execution) {
    info!("开始监控工作流执行");

    // 记录执行开始指标
    counter!("workflow_executions_started").increment(1);
    gauge!("active_executions").increment(1.0);

    // 开始追踪
    // self.tracer.start_execution_trace(execution).await;
  }

  pub async fn record_node_execution(
    &self,
    node_name: &NodeName,
    duration: std::time::Duration,
    result: &Result<Vec<VecExecutionData>, NodeExecutionError>,
  ) {
    let duration_ms = duration.as_millis() as f64;

    // 记录执行时间
    histogram!("node_execution_duration_ms").record(duration_ms);

    // 记录执行结果
    match result {
      Ok(data) => {
        counter!("node_executions_success").increment(1);
        histogram!("node_output_size").record(data.len() as f64);

        debug!("节点执行成功");
      }
      Err(error) => {
        counter!("node_executions_failed").increment(1);

        error!("节点执行失败");
      }
    }
  }

  pub async fn stop_execution_monitoring(&self, execution_id: &ExecutionId) {
    gauge!("active_executions").decrement(1.0);

    info!("停止监控工作流执行");

    // 结束追踪
    // self.tracer.end_execution_trace(execution_id).await;
  }
}
