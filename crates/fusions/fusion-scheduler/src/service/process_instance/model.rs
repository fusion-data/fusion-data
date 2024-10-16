use fusion_scheduler_api::v1::process_instance::InstanceStatus;
use modql::field::Fields;
use sea_query::enum_def;
use sqlx::prelude::FromRow;
use ultimate_common::time::UtcDateTime;
use uuid::Uuid;

#[derive(FromRow, Fields)]
#[enum_def]
pub struct ProcessInstance {
  pub id: Uuid,
  pub process_id: i64,
  pub trigger_id: i64,
  pub status: InstanceStatus,
  pub retry_count: i32,
  pub execution_time: UtcDateTime,
  pub complete_time: Option<UtcDateTime>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}

#[derive(Default, Fields)]
pub struct ProcessInstanceForCreate {}

#[derive(Default, Fields)]
pub struct ProcessInstanceForUpdate {}
