use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt64, OpValsValue},
};
use sqlx::prelude::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::{to_sea_chrono_utc, uuid_to_sea_value, DbRowType};
use uuid::Uuid;

#[derive(Debug, FromRow, Fields)]
pub struct JobTrigger {
  pub job_id: Uuid,
  pub trigger_id: Uuid,
  pub cid: i64,
  pub ctime: UtcDateTime,
}
impl DbRowType for JobTrigger {}

#[derive(Debug, Fields)]
pub struct JobTriggerForCreate {
  pub job_id: Uuid,
  pub trigger_id: Uuid,
}

#[derive(Debug, Default, FilterNodes)]
pub struct JobTriggerFilter {
  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub job_id: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub trigger_id: Option<OpValsValue>,

  pub cid: Option<OpValsInt64>,

  #[modql(to_sea_value_fn = "to_sea_chrono_utc")]
  pub ctime: Option<OpValsValue>,
}
