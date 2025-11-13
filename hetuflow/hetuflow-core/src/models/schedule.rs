use chrono::{DateTime, FixedOffset};
use fusion_common::time::now_offset;
use fusionsql_core::page::Page;
use fusionsql_core::{
  field::FieldMask,
  filter::{OpValDateTime, OpValInt32, OpValUuid},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{ScheduleKind, ScheduleStatus};

/// SchedSchedule 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "with-db",
  derive(fusionsql::Fields, sqlx::FromRow),
  sea_query::enum_def(table_name = "sched_schedule")
)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedSchedule {
  pub id: Uuid,
  pub job_id: Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  pub schedule_kind: ScheduleKind,
  pub start_time: Option<DateTime<FixedOffset>>,
  pub end_time: Option<DateTime<FixedOffset>>,
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
  pub next_run_at: Option<DateTime<FixedOffset>>,

  pub created_by: i64,
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}

impl SchedSchedule {
  pub fn is_valid(&self) -> bool {
    let now = now_offset();
    self.status == ScheduleStatus::Enabled
      && self.start_time.map(|start_time| start_time <= now).unwrap_or(true)
      && self.end_time.map(|end_time| end_time >= now).unwrap_or(true)
  }
}

/// Schedule 创建模型
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ScheduleForCreate {
  pub id: Uuid,
  pub job_id: Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  pub schedule_kind: ScheduleKind,
  pub cron_expression: Option<String>,
  pub start_time: Option<DateTime<FixedOffset>>,
  pub end_time: Option<DateTime<FixedOffset>>,
  pub status: Option<ScheduleStatus>,
}

/// Schedule 更新模型
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ScheduleForUpdate {
  pub name: Option<String>,
  pub description: Option<String>,
  pub schedule_kind: Option<ScheduleKind>,
  pub cron_expression: Option<String>,
  pub start_time: Option<DateTime<FixedOffset>>,
  pub end_time: Option<DateTime<FixedOffset>>,
  pub status: Option<ScheduleStatus>,
  pub update_mask: Option<FieldMask>,
}

/// Schedule 过滤器
#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ScheduleFilter {
  pub id: Option<OpValUuid>,
  pub job_id: Option<OpValUuid>,
  pub schedule_kind: Option<OpValInt32>,
  pub status: Option<OpValInt32>,
  pub created_at: Option<OpValDateTime>,
  pub updated_at: Option<OpValDateTime>,
}

#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ScheduleForQuery {
  pub page: Page,
  pub filter: ScheduleFilter,
}
