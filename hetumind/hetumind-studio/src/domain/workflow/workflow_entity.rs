use hetumind_core::workflow::{Workflow, WorkflowId, WorkflowStatus};
use modelsql::{field::Fields, postgres::PgRowType};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::OffsetDateTime;
use ultimate_core::DataError;

/// 工作流实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "workflow_entity")]
pub struct WorkflowEntity {
  pub id: WorkflowId,
  pub name: String,
  pub status: WorkflowStatus,
  pub version_id: Option<WorkflowId>,
  pub settings: serde_json::Value,
  pub meta: serde_json::Value,
  /// Vec<WorkflowNode>
  pub nodes: serde_json::Value,
  /// Vec<Connection>
  pub connections: serde_json::Value,
  /// [PinData]
  pub pin_data: serde_json::Value,
  pub static_data: Option<serde_json::Value>,
  pub parent_folder_id: Option<String>,
  pub trigger_count: i64,
  pub is_archived: bool,
  pub created_at: OffsetDateTime,
  pub created_by: i64,
  pub updated_at: Option<OffsetDateTime>,
  pub updated_by: Option<i64>,
}
impl PgRowType for WorkflowEntity {}

impl TryFrom<WorkflowEntity> for Workflow {
  type Error = DataError;

  fn try_from(entity: WorkflowEntity) -> Result<Self, Self::Error> {
    let wf = Self {
      id: entity.id,
      name: entity.name,
      status: entity.status,
      version: entity.version_id,
      settings: serde_json::from_value(entity.settings)?,
      meta: serde_json::from_value(entity.meta)?,
      nodes: serde_json::from_value(entity.nodes)?,
      connections: serde_json::from_value(entity.connections)?,
      pin_data: serde_json::from_value(entity.pin_data)?,
      static_data: if let Some(static_data) = entity.static_data {
        Some(serde_json::from_value(static_data)?)
      } else {
        None
      },
    };
    Ok(wf)
  }
}

/// 共享工作流表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "shared_workflow")]
pub struct SharedWorkflow {
  pub workflow_id: String,
  pub project_id: String,
  pub role: String,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
}

/// 工作流历史表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "workflow_history")]
pub struct WorkflowHistory {
  pub version_id: String,
  pub workflow_id: String,
  pub authors: String,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
  pub nodes: serde_json::Value,
  pub connections: serde_json::Value,
}

/// 工作流统计表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "workflow_statistics")]
pub struct WorkflowStatistics {
  pub count: Option<i32>,
  pub latest_event: Option<OffsetDateTime>,
  pub name: String,
  pub workflow_id: String,
  pub root_count: Option<i32>,
}

/// 处理数据表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "processed_data")]
pub struct ProcessedData {
  pub workflow_id: String,
  pub context: String,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
  pub value: String,
}
