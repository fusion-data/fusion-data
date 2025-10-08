use chrono::{DateTime, FixedOffset};
use fusion_common::page::Page;
use fusionsql_core::{
  field::FieldMask,
  filter::{OpValDateTime, OpValInt32, OpValString, OpValUuid},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::TaskInstanceStatus;

use super::TaskMetrics;

/// SchedTaskInstance 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields, sqlx::FromRow))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedTaskInstance {
  pub id: Uuid,
  pub task_id: Uuid,
  pub job_id: Uuid,
  pub agent_id: Option<String>,
  pub status: TaskInstanceStatus,
  pub started_at: Option<DateTime<FixedOffset>>,
  pub completed_at: Option<DateTime<FixedOffset>>,
  pub output: Option<String>,
  pub error_message: Option<String>,
  pub exit_code: Option<i32>,
  pub metrics: Option<TaskMetrics>,
  pub created_at: DateTime<FixedOffset>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}

/// TaskInstance 创建模型
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskInstanceForCreate {
  pub id: Option<Uuid>,
  pub job_id: Uuid,
  pub task_id: Uuid,
  pub agent_id: Option<String>,
  pub status: TaskInstanceStatus,
  pub started_at: Option<DateTime<FixedOffset>>,
}

/// TaskInstance 更新模型
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskInstanceForUpdate {
  pub agent_id: Option<String>,
  pub status: Option<TaskInstanceStatus>,
  pub started_at: Option<DateTime<FixedOffset>>,
  pub completed_at: Option<DateTime<FixedOffset>>,
  pub output: Option<String>,
  pub error_message: Option<String>,
  pub exit_code: Option<i32>,
  pub metrics: Option<TaskMetrics>,
  pub update_mask: Option<FieldMask>,
}

/// TaskInstance 查询请求
#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskInstanceForQuery {
  pub filter: TaskInstanceFilter,
  pub page: Page,
}

/// TaskInstance 过滤器
#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskInstanceFilter {
  pub id: Option<OpValUuid>,
  pub task_id: Option<OpValUuid>,
  pub agent_id: Option<OpValString>,
  pub status: Option<OpValInt32>,
  pub started_at: Option<OpValDateTime>,
  pub completed_at: Option<OpValDateTime>,
  pub created_at: Option<OpValDateTime>,
  pub updated_at: Option<OpValDateTime>,
}

/// 任务状态信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskStatusInfo {
  pub task_id: Uuid,              // 任务ID
  pub status: TaskInstanceStatus, // 执行状态
  pub agent_id: String,           // Agent ID
  pub start_time: Option<i64>,    // 开始时间
  pub progress: Option<f64>,      // 执行进度 (0.0-1.0)
}
