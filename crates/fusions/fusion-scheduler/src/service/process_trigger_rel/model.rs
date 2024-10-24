use sqlx::prelude::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt64, OpValsValue},
};
use ultimate_db::{datetime_to_sea_value, DbRowType};

#[derive(Debug, FromRow, Fields)]
pub struct ProcessTriggerRel {
  pub process_id: i64,
  pub trigger_id: i64,
  pub cid: i64,
  pub ctime: UtcDateTime,
}
impl DbRowType for ProcessTriggerRel {}

#[derive(Debug, Fields)]
pub struct ProcessTriggerRelForCreate {
  pub process_id: i64,
  pub trigger_id: i64,
}

#[derive(Debug, Default, FilterNodes)]
pub struct ProcessTriggerRelFilter {
  pub process_id: Option<OpValsInt64>,

  pub trigger_id: Option<OpValsInt64>,

  pub cid: Option<OpValsInt64>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,
}
