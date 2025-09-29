use ahash::HashMap;
use fusion_common::time::OffsetDateTime;
use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsUuid, Page},
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use typed_builder::TypedBuilder;

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
modelsql::generate_enum_i32_to_sea_query_value!(Enum: ExecutionStatus, Enum: ExecutionMode);

#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct Execution {
  /// 执行唯一标识符
  pub id: ExecutionId,
  /// 所属工作流ID
  pub workflow_id: WorkflowId,
  /// 执行状态
  #[builder(default)]
  pub status: ExecutionStatus,
  /// 开始时间
  #[builder(default, setter(strip_option))]
  pub started_at: Option<OffsetDateTime>,
  /// 结束时间
  #[builder(default, setter(strip_option))]
  pub finished_at: Option<OffsetDateTime>,
  /// 执行数据
  #[builder(default, setter(strip_option))]
  pub data: Option<serde_json::Value>,
  /// 错误信息
  #[builder(default, setter(strip_option))]
  pub error: Option<String>,
  /// 执行模式
  #[builder(default)]
  pub mode: ExecutionMode,
  /// 触发者ID
  #[builder(default, setter(strip_option))]
  pub triggered_by: Option<UserId>,
}

/// 工作流执行结果
#[derive(Debug, TypedBuilder)]
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
  pub fn is_success(&self) -> bool {
    self.status == ExecutionStatus::Success
  }

  pub fn output_data(&self) -> Vec<&NodeExecutionResult> {
    self.nodes_result.values().filter(|result| self.end_nodes.contains(&result.node_name)).collect()
  }
}

#[derive(Deserialize)]
#[cfg_attr(feature = "modelsql", derive(modelsql::field::Fields))]
pub struct ExecutionForUpdate {
  pub status: Option<ExecutionStatus>,
  pub finished_at: Option<OffsetDateTime>,
  pub wait_till: Option<OffsetDateTime>,
  pub retry_of: Option<ExecutionId>,
  pub retry_success_id: Option<ExecutionId>,
  pub started_at: Option<OffsetDateTime>,
  pub deleted_at: Option<OffsetDateTime>,
  #[cfg_attr(feature = "modelsql", field(skip))]
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
      deleted_at: None,
      field_mask: None,
    }
  }
}

#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "modelsql", derive(modelsql::filter::FilterNodes))]
pub struct ExecutionFilter {
  pub workflow_id: Option<OpValsUuid>,
  pub status: Option<OpValsInt32>,
  pub started_at: Option<OpValsDateTime>,
  pub finished_at: Option<OpValsDateTime>,
  pub wait_till: Option<OpValsDateTime>,
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
