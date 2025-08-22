use modelsql::{
  Fields, FilterNodes,
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt64, OpValsString, Page},
};
use sea_query::enum_def;
use serde::Deserialize;
use sqlx::FromRow;
use ultimate_common::time::OffsetDateTime;

pub struct DistributedLockIds;
impl DistributedLockIds {
  pub const SCHED_SERVER_LEADER: &str = "sched_server_leader";
}

#[derive(Debug, FromRow, Fields)]
#[enum_def(table_name = "distributed_lock")]
pub struct DistributedLockEntity {
  pub id: String,
  pub value: String,
  pub locked_at: OffsetDateTime,
  pub expires_at: OffsetDateTime,
  pub token: i64,
}

#[derive(Deserialize, Fields)]
pub struct DistributedLockForInsert {
  pub id: String,
  pub value: String,
}

#[derive(Deserialize, Default, Fields)]
pub struct DistributedLockForUpdate {
  pub value: Option<String>,
  pub locked_at: Option<OffsetDateTime>,
  pub expires_at: Option<OffsetDateTime>,
  pub token: Option<i64>,
  pub mask: FieldMask,
}

#[derive(Default, Deserialize, FilterNodes)]
pub struct DistributedLockFilter {
  pub id: Option<OpValsString>,
  pub value: Option<OpValsString>,
  pub locked_at: Option<OpValsDateTime>,
  pub expires_at: Option<OpValsDateTime>,
  pub token: Option<OpValsInt64>,
}

#[derive(Deserialize)]
pub struct DistributedLockForQuery {
  pub filter: DistributedLockFilter,
  pub page: Page,
}
