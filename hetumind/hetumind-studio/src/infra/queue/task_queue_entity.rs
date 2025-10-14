use fusion_common::time::OffsetDateTime;
use fusionsql::{field::Fields, postgres::PgRowType};
use hetumind_core::{
  task::{TaskPriority, TaskStatus},
  workflow::{ExecutionId, WorkflowId},
};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "task_queue")]
pub struct TaskQueueEntity {
  pub id: Uuid,
  pub task_kind: i32,
  pub execution_id: ExecutionId,
  pub workflow_id: WorkflowId,
  pub priority: TaskPriority,
  pub status: TaskStatus,
  pub payload: serde_json::Value,
  pub result: Option<serde_json::Value>,
  pub error_message: Option<String>,
  pub retry_count: i32,
  pub max_retries: i32,
  pub scheduled_at: OffsetDateTime,
  pub started_at: Option<OffsetDateTime>,
  pub completed_at: Option<OffsetDateTime>,
  pub worker_id: Option<Uuid>,
  pub heartbeat_at: Option<OffsetDateTime>,
  pub metadata: serde_json::Value,
  pub created_at: OffsetDateTime,
  pub created_by: i64,
  pub updated_at: Option<OffsetDateTime>,
  pub updated_by: Option<i64>,
}
impl PgRowType for TaskQueueEntity {}
