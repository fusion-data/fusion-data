use sqlx::prelude::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt64, OpValsUuid, OpValsValue},
};
use ultimate_db::{datetime_to_sea_value, DbRowType};
use uuid::Uuid;

#[derive(Debug, FromRow, Fields)]
pub struct ProcessTriggerRel {
  pub process_id: Uuid,
  pub trigger_id: Uuid,
  pub cid: i64,
  pub ctime: UtcDateTime,
}
impl DbRowType for ProcessTriggerRel {}

#[derive(Debug, Fields)]
pub struct ProcessTriggerRelForCreate {
  pub process_id: Uuid,
  pub trigger_id: Uuid,
}

#[derive(Debug, Default, FilterNodes)]
pub struct ProcessTriggerRelFilter {
  pub process_id: Option<OpValsUuid>,

  pub trigger_id: Option<OpValsUuid>,

  pub cid: Option<OpValsInt64>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,
}
