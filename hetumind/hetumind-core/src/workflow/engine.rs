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

impl ExecutionMetrics {
  pub fn new(
    execution_id: ExecutionId,
    duration_ms: u64,
    nodes_executed: u32,
    nodes_succeeded: u32,
    nodes_failed: u32,
  ) -> Self {
    Self {
      execution_id,
      duration_ms,
      nodes_executed,
      nodes_succeeded,
      nodes_failed,
      memory_usage_mb: Default::default(),
      cpu_usage_percent: Default::default(),
      cache_hit_rate: Default::default(),
      retry_count: Default::default(),
    }
  }

  pub fn with_execution_id(mut self, execution_id: ExecutionId) -> Self {
    self.execution_id = execution_id;
    self
  }

  pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
    self.duration_ms = duration_ms;
    self
  }

  pub fn with_nodes_executed(mut self, nodes_executed: u32) -> Self {
    self.nodes_executed = nodes_executed;
    self
  }

  pub fn with_nodes_succeeded(mut self, nodes_succeeded: u32) -> Self {
    self.nodes_succeeded = nodes_succeeded;
    self
  }

  pub fn with_nodes_failed(mut self, nodes_failed: u32) -> Self {
    self.nodes_failed = nodes_failed;
    self
  }

  pub fn with_memory_usage_mb(mut self, memory_usage_mb: f64) -> Self {
    self.memory_usage_mb = memory_usage_mb;
    self
  }

  pub fn with_cpu_usage_percent(mut self, cpu_usage_percent: f64) -> Self {
    self.cpu_usage_percent = cpu_usage_percent;
    self
  }

  pub fn with_cache_hit_rate(mut self, cache_hit_rate: f64) -> Self {
    self.cache_hit_rate = cache_hit_rate;
    self
  }

  pub fn with_retry_count(mut self, retry_count: u32) -> Self {
    self.retry_count = retry_count;
    self
  }
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

impl ExecutionTrace {
  pub fn new(execution_id: ExecutionId, start_time: chrono::DateTime<chrono::FixedOffset>) -> Self {
    Self { execution_id, start_time, end_time: None, node_traces: Vec::default(), error_traces: Vec::default() }
  }

  pub fn with_execution_id(mut self, execution_id: ExecutionId) -> Self {
    self.execution_id = execution_id;
    self
  }

  pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::FixedOffset>) -> Self {
    self.start_time = start_time;
    self
  }

  pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::FixedOffset>) -> Self {
    self.end_time = Some(end_time);
    self
  }

  pub fn with_node_traces<I, V>(mut self, node_traces: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<NodeTrace>,
  {
    self.node_traces = node_traces.into_iter().map(|v| v.into()).collect();
    self
  }

  pub fn add_node_trace(mut self, node_trace: impl Into<NodeTrace>) -> Self {
    self.node_traces.push(node_trace.into());
    self
  }

  pub fn with_error_traces<I, V>(mut self, error_traces: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<ErrorTrace>,
  {
    self.error_traces = error_traces.into_iter().map(|v| v.into()).collect();
    self
  }

  pub fn add_error_trace(mut self, error_trace: impl Into<ErrorTrace>) -> Self {
    self.error_traces.push(error_trace.into());
    self
  }
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

impl NodeTrace {
  pub fn new(
    node_name: NodeName,
    start_time: chrono::DateTime<chrono::FixedOffset>,
    status: NodeExecutionStatus,
    input_size_bytes: u64,
    output_size_bytes: u64,
  ) -> Self {
    Self {
      node_name,
      start_time,
      end_time: None,
      status,
      input_size_bytes,
      output_size_bytes,
      memory_peak_mb: Default::default(),
    }
  }

  pub fn with_node_name(mut self, node_name: NodeName) -> Self {
    self.node_name = node_name;
    self
  }

  pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::FixedOffset>) -> Self {
    self.start_time = start_time;
    self
  }

  pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::FixedOffset>) -> Self {
    self.end_time = Some(end_time);
    self
  }

  pub fn with_status(mut self, status: NodeExecutionStatus) -> Self {
    self.status = status;
    self
  }

  pub fn with_input_size_bytes(mut self, input_size_bytes: u64) -> Self {
    self.input_size_bytes = input_size_bytes;
    self
  }

  pub fn with_output_size_bytes(mut self, output_size_bytes: u64) -> Self {
    self.output_size_bytes = output_size_bytes;
    self
  }

  pub fn with_memory_peak_mb(mut self, memory_peak_mb: f64) -> Self {
    self.memory_peak_mb = memory_peak_mb;
    self
  }
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

impl ErrorTrace {
  pub fn new(
    node_name: NodeName,
    error_time: chrono::DateTime<chrono::FixedOffset>,
    error_type: impl Into<String>,
    error_message: impl Into<String>,
  ) -> Self {
    Self {
      node_name,
      error_time,
      error_type: error_type.into(),
      error_message: error_message.into(),
      stack_trace: None,
    }
  }

  pub fn with_node_name(mut self, node_name: NodeName) -> Self {
    self.node_name = node_name;
    self
  }

  pub fn with_error_time(mut self, error_time: chrono::DateTime<chrono::FixedOffset>) -> Self {
    self.error_time = error_time;
    self
  }

  pub fn with_error_type(mut self, error_type: impl Into<String>) -> Self {
    self.error_type = error_type.into();
    self
  }

  pub fn with_error_message(mut self, error_message: impl Into<String>) -> Self {
    self.error_message = error_message.into();
    self
  }

  pub fn with_stack_trace(mut self, stack_trace: impl Into<String>) -> Self {
    self.stack_trace = Some(stack_trace.into());
    self
  }
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
