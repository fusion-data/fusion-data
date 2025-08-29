use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsUuid, Page},
};
use serde::{Deserialize, Serialize};
use ultimate_common::time::OffsetDateTime;
use uuid::Uuid;

use crate::types::TaskInstanceStatus;

/// TaskInstanceEntity 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "with-db",
  derive(modelsql::Fields, sqlx::FromRow),
  sea_query::enum_def(table_name = "sched_task_instance")
)]
pub struct TaskInstanceEntity {
  pub id: Uuid,
  pub task_id: Uuid,
  pub job_id: Uuid,
  pub agent_id: Uuid,
  pub status: TaskInstanceStatus,
  pub started_at: OffsetDateTime,
  pub completed_at: Option<OffsetDateTime>,
  pub output: Option<String>,
  pub error_message: Option<String>,
  pub exit_code: Option<i32>,
  pub metrics: Option<serde_json::Value>,
  pub created_at: OffsetDateTime,
  pub updated_by: Option<i64>,
  pub updated_at: Option<OffsetDateTime>,
}

/// TaskInstance 创建模型
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
pub struct TaskInstanceForCreate {
  pub id: Option<Uuid>,
  pub task_id: Uuid,
  pub server_id: Option<Uuid>,
  pub agent_id: Option<Uuid>,
  pub status: TaskInstanceStatus,
}

/// TaskInstance 更新模型
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
pub struct TaskInstanceForUpdate {
  pub server_id: Option<Uuid>,
  pub agent_id: Option<Uuid>,
  pub status: Option<TaskInstanceStatus>,
  pub started_at: Option<OffsetDateTime>,
  pub completed_at: Option<OffsetDateTime>,
  pub output: Option<String>,
  pub error_message: Option<String>,
  pub exit_code: Option<i32>,
  pub metrics: Option<serde_json::Value>,
  pub update_mask: Option<FieldMask>,
}

/// TaskInstance 查询请求
#[derive(Default, Deserialize)]
pub struct TaskInstanceForQuery {
  pub filter: TaskInstanceFilter,
  pub page: Page,
}

/// TaskInstance 过滤器
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::FilterNodes))]
pub struct TaskInstanceFilter {
  pub id: Option<OpValsUuid>,
  pub task_id: Option<OpValsUuid>,
  pub server_id: Option<OpValsUuid>,
  pub agent_id: Option<OpValsUuid>,
  pub status: Option<OpValsInt32>,
  pub started_at: Option<OpValsDateTime>,
  pub completed_at: Option<OpValsDateTime>,
  pub created_at: Option<OpValsDateTime>,
  pub updated_at: Option<OpValsDateTime>,
}

/// 任务状态信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskStatusInfo {
  pub task_id: Uuid,              // 任务ID
  pub status: TaskInstanceStatus, // 执行状态
  pub agent_id: Uuid,             // Agent ID
  pub start_time: Option<i64>,    // 开始时间
  pub progress: Option<f64>,      // 执行进度 (0.0-1.0)
}
