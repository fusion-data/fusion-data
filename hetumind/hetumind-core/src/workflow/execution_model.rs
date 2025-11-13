use fusion_common::ahash::HashMap;
use fusion_common::time::OffsetDateTime;
use fusionsql_core::page::Page;
use fusionsql_core::{
  field::FieldMask,
  filter::{OpValDateTime, OpValInt32, OpValUuid},
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::user::UserId;

use super::{ExecutionId, NodeExecutionResult, NodeName, ParameterMap, WorkflowId};

/// 执行模式
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum ExecutionMode {
  /// 单机执行模式
  #[default]
  Local = 1,
  /// 分布式执行模式
  Distributed = 2,
}

/// 工作流执行状态
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum ExecutionStatus {
  /// 新建，等待执行
  #[default]
  New = 1,

  /// 执行中
  Running = 10,

  /// 等待外部事件（如 webhook、定时器）
  Waiting = 11,

  /// 重试中
  Retrying = 21,

  /// 执行已取消
  Cancelled = 97,

  /// 执行崩溃
  Crashed = 98,

  /// 执行错误
  Failed = 99,

  /// 执行成功
  Success = 100,
}

#[cfg(feature = "with-db")]
fusionsql::generate_enum_i32_to_sea_query_value!(Enum: ExecutionStatus, Enum: ExecutionMode);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
  /// 执行唯一标识符
  pub id: ExecutionId,
  /// 所属工作流ID
  pub workflow_id: WorkflowId,
  /// 执行状态
  #[serde(default)]
  pub status: ExecutionStatus,
  /// 开始时间
  pub started_at: Option<OffsetDateTime>,
  /// 结束时间
  pub finished_at: Option<OffsetDateTime>,
  /// 执行数据
  pub data: Option<serde_json::Value>,
  /// 错误信息
  pub error: Option<String>,
  /// 执行模式
  #[serde(default)]
  pub mode: ExecutionMode,
  /// 触发者ID
  pub triggered_by: Option<UserId>,
}

impl Execution {
  pub fn new(id: ExecutionId, workflow_id: WorkflowId) -> Self {
    Self {
      id,
      workflow_id,
      status: ExecutionStatus::default(),
      started_at: None,
      finished_at: None,
      data: None,
      error: None,
      mode: ExecutionMode::default(),
      triggered_by: None,
    }
  }

  pub fn with_status(mut self, status: ExecutionStatus) -> Self {
    self.status = status;
    self
  }

  pub fn with_started_at(mut self, started_at: OffsetDateTime) -> Self {
    self.started_at = Some(started_at);
    self
  }

  pub fn with_finished_at(mut self, finished_at: OffsetDateTime) -> Self {
    self.finished_at = Some(finished_at);
    self
  }

  pub fn with_data(mut self, data: impl Into<serde_json::Value>) -> Self {
    self.data = Some(data.into());
    self
  }

  pub fn with_error(mut self, error: impl Into<String>) -> Self {
    self.error = Some(error.into());
    self
  }

  pub fn with_mode(mut self, mode: ExecutionMode) -> Self {
    self.mode = mode;
    self
  }

  pub fn with_triggered_by(mut self, triggered_by: UserId) -> Self {
    self.triggered_by = Some(triggered_by);
    self
  }
}

/// 工作流执行结果
#[derive(Debug)]
pub struct ExecutionResult {
  /// 执行ID
  pub execution_id: ExecutionId,

  /// 最终状态
  pub status: ExecutionStatus,

  /// 结束节点
  pub end_nodes: Vec<NodeName>,

  /// 执行时长
  pub duration_ms: u64,

  /// 所有节点执行结果（可选）
  pub nodes_result: HashMap<NodeName, NodeExecutionResult>,
}

impl ExecutionResult {
  pub fn new(
    execution_id: ExecutionId,
    status: ExecutionStatus,
    end_nodes: Vec<NodeName>,
    duration_ms: u64,
    nodes_result: HashMap<NodeName, NodeExecutionResult>,
  ) -> Self {
    Self { execution_id, status, end_nodes, duration_ms, nodes_result }
  }

  pub fn is_success(&self) -> bool {
    self.status == ExecutionStatus::Success
  }

  pub fn output_data(&self) -> Vec<&NodeExecutionResult> {
    self.nodes_result.values().filter(|result| self.end_nodes.contains(&result.node_name)).collect()
  }
}

#[derive(Deserialize)]
#[cfg_attr(feature = "fusionsql", derive(fusionsql::Fields))]
pub struct ExecutionForUpdate {
  pub status: Option<ExecutionStatus>,
  pub finished_at: Option<OffsetDateTime>,
  pub wait_till: Option<OffsetDateTime>,
  pub retry_of: Option<ExecutionId>,
  pub retry_success_id: Option<ExecutionId>,
  pub started_at: Option<OffsetDateTime>,
  pub logical_deletion: Option<OffsetDateTime>,
  #[cfg_attr(feature = "fusionsql", field(skip))]
  pub field_mask: Option<FieldMask>,
}

impl From<Execution> for ExecutionForUpdate {
  fn from(execution: Execution) -> Self {
    ExecutionForUpdate {
      status: Some(execution.status),
      finished_at: execution.finished_at,
      wait_till: None,
      retry_of: None,
      retry_success_id: None,
      started_at: execution.started_at,
      logical_deletion: None,
      field_mask: None,
    }
  }
}

#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "fusionsql", derive(fusionsql::filter::FilterNodes))]
pub struct ExecutionFilter {
  pub workflow_id: Option<OpValUuid>,
  pub status: Option<OpValInt32>,
  pub started_at: Option<OpValDateTime>,
  pub finished_at: Option<OpValDateTime>,
  pub wait_till: Option<OpValDateTime>,
}

#[derive(Serialize, Deserialize)]
pub struct ExecutionForQuery {
  pub options: Page,
  pub filter: ExecutionFilter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteWorkflowRequest {
  pub input_data: Option<ParameterMap>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionIdResponse {
  pub execution_id: ExecutionId,
}
