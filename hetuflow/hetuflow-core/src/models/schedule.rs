use fusion_common::time::OffsetDateTime;
use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsUuid},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{ScheduleKind, ScheduleStatus};

/// SchedSchedule 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "with-db",
  derive(modelsql::Fields, sqlx::FromRow),
  sea_query::enum_def(table_name = "sched_schedule")
)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedSchedule {
  pub id: Uuid,
  pub job_id: Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  pub schedule_kind: ScheduleKind,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub start_time: Option<OffsetDateTime>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub end_time: Option<OffsetDateTime>,
  pub status: ScheduleStatus,

  /// ScheduleKind::Cron 时有效
  pub cron_expression: Option<String>,

  /// ScheduleKind::Interval 时有效
  /// 间隔时间，单位秒
  pub interval_secs: Option<i32>,

  /// ScheduleKind::Interval 时有效
  /// 最大执行次数，为 1 时表示只执行一次，为 None 时表示无限执行。
  pub max_count: Option<i32>,

  /// 计算出的下一次执行时间
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub next_run_at: Option<OffsetDateTime>,

  pub created_by: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub created_at: OffsetDateTime,
  pub updated_by: Option<i64>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub updated_at: Option<OffsetDateTime>,
}

/// Schedule 创建模型
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ScheduleForCreate {
  pub id: Uuid,
  pub job_id: Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  pub schedule_kind: String,
  pub cron_expression: Option<String>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub start_time: Option<OffsetDateTime>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub end_time: Option<OffsetDateTime>,
  pub status: Option<ScheduleStatus>,
}

/// Schedule 更新模型
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ScheduleForUpdate {
  pub name: Option<String>,
  pub description: Option<String>,
  pub schedule_kind: Option<String>,
  pub cron_expression: Option<String>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub start_time: Option<OffsetDateTime>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub end_time: Option<OffsetDateTime>,
  pub status: Option<ScheduleStatus>,
  pub update_mask: Option<FieldMask>,
}

/// Schedule 过滤器
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ScheduleFilter {
  pub id: Option<OpValsUuid>,
  pub job_id: Option<OpValsUuid>,
  pub schedule_kind: Option<OpValsInt32>,
  pub status: Option<OpValsInt32>,
  pub created_at: Option<OpValsDateTime>,
  pub updated_at: Option<OpValsDateTime>,
}
