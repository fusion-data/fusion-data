use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt64, OpValsUuid, OpValsValue},
  postgres::PgRowType,
  utils::datetime_to_sea_value,
};
use sqlx::prelude::FromRow;
use ultimate_common::time::UtcDateTime;
use uuid::Uuid;

#[derive(Debug, FromRow, Fields)]
pub struct ProcessTriggerRel {
  pub process_id: Uuid,
  pub trigger_id: Uuid,
  pub cid: i64,
  pub ctime: UtcDateTime,
}
impl PgRowType for ProcessTriggerRel {}

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

  #[modelsql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,
}
