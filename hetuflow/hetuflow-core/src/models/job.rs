use chrono::{DateTime, FixedOffset};
use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsString, OpValsUuid, Page},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::JobStatus;

use super::TaskConfig;

/// SchedJob 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields, sqlx::FromRow), sea_query::enum_def(table_name = "sched_job"))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedJob {
  pub id: Uuid,
  pub namespace_id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub environment: Option<serde_json::Value>,
  pub config: TaskConfig,
  pub status: JobStatus,
  pub created_at: DateTime<FixedOffset>,
  pub last_heartbeat_at: Option<DateTime<FixedOffset>>,
}

/// Job 创建模型
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct JobForCreate {
  pub id: Option<Uuid>,
  pub namespace_id: Option<Uuid>,
  pub name: String,
  pub description: Option<String>,
  pub environment: Option<serde_json::Value>,
  pub config: Option<TaskConfig>,
  pub status: Option<JobStatus>,
}

/// Job 更新模型
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct JobForUpdate {
  pub namespace_id: Option<Uuid>,
  pub name: Option<String>,
  pub description: Option<String>,
  pub command: Option<String>,
  pub environment: Option<serde_json::Value>,
  pub config: Option<serde_json::Value>,
  pub status: Option<JobStatus>,
  pub update_mask: Option<FieldMask>,
}

/// Job 查询请求
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct JobForQuery {
  pub filter: JobFilter,
  pub page: Page,
}

/// Job 过滤器
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct JobFilter {
  pub id: Option<OpValsUuid>,
  pub name: Option<OpValsString>,
  pub namespace_id: Option<OpValsUuid>,
  pub status: Option<OpValsInt32>,
  pub created_at: Option<OpValsDateTime>,
  pub last_heartbeat_at: Option<OpValsDateTime>,
}
