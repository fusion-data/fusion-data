use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{
  ExecutionContext, ExecutionId, ExecutionResult, ExecutionStatus, NodeName, WorkflowExecutionError,
  WorkflowTriggerData,
};

/// 执行指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
  pub execution_id: ExecutionId,
  pub duration_ms: u64,
  pub nodes_executed: u32,
  pub nodes_succeeded: u32,
  pub nodes_failed: u32,
  pub memory_usage_mb: f64,
  pub cpu_usage_percent: f64,
  pub cache_hit_rate: f64,
  pub retry_count: u32,
}

/// 执行追踪
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
  pub execution_id: ExecutionId,
  pub start_time: chrono::DateTime<chrono::FixedOffset>,
  pub end_time: Option<chrono::DateTime<chrono::FixedOffset>>,
  pub node_traces: Vec<NodeTrace>,
  pub error_traces: Vec<ErrorTrace>,
}

/// 节点执行追踪
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTrace {
  pub node_name: NodeName,
  pub start_time: chrono::DateTime<chrono::FixedOffset>,
  pub end_time: Option<chrono::DateTime<chrono::FixedOffset>>,
  pub status: NodeExecutionStatus,
  pub input_size_bytes: u64,
  pub output_size_bytes: u64,
  pub memory_peak_mb: f64,
}

/// 错误追踪
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrace {
  pub node_name: NodeName,
  pub error_time: chrono::DateTime<chrono::FixedOffset>,
  pub error_type: String,
  pub error_message: String,
  pub stack_trace: Option<String>,
}

use crate::workflow::NodeExecutionStatus;

#[async_trait]
pub trait WorkflowEngine: Send + Sync {
  /// 执行工作流（新的统一接口）
  async fn execute_workflow(
    &self,
    trigger_data: WorkflowTriggerData,
    context: &ExecutionContext,
  ) -> Result<ExecutionResult, WorkflowExecutionError>;

  /// 暂停执行
  async fn pause_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError>;

  /// 恢复执行
  async fn resume_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError>;

  /// 取消执行
  async fn cancel_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError>;

  /// 获取执行状态
  async fn get_execution_status(&self, execution_id: &ExecutionId) -> Result<ExecutionStatus, WorkflowExecutionError>;

  // 新增优化方法（可选实现）
  /// 获取执行指标
  async fn get_execution_metrics(
    &self,
    _execution_id: &ExecutionId,
  ) -> Result<Option<ExecutionMetrics>, WorkflowExecutionError> {
    Ok(None) // 默认实现返回空
  }

  /// 获取执行追踪
  async fn get_execution_trace(
    &self,
    _execution_id: &ExecutionId,
  ) -> Result<Option<ExecutionTrace>, WorkflowExecutionError> {
    Ok(None) // 默认实现返回空
  }

  /// 启用/禁用并行执行
  async fn set_parallel_execution(&self, _enabled: bool) -> Result<(), WorkflowExecutionError> {
    // 默认实现不支持动态配置
    Err(WorkflowExecutionError::InvalidWorkflowStructure(
      "Dynamic parallel execution configuration not supported".to_string(),
    ))
  }

  /// 启用/禁用节点缓存
  async fn set_node_caching(&self, _enabled: bool) -> Result<(), WorkflowExecutionError> {
    // 默认实现不支持动态配置
    Err(WorkflowExecutionError::InvalidWorkflowStructure(
      "Dynamic node caching configuration not supported".to_string(),
    ))
  }
}
