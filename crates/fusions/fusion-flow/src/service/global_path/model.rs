use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt64, OpValsString},
  postgres::PgRowType,
};
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow, Fields)]
pub struct GlobalPath {
  pub path: String,
  pub value: Option<String>,
  pub revision: i64,
}
impl PgRowType for GlobalPath {}

impl GlobalPath {
  pub const MASTER: &str = "/master";
  pub const NODE_TRIGGER: &str = "/node/trigger";
}

#[derive(Default, FilterNodes)]
pub struct GlobalPathFilter {
  pub path: Option<OpValsString>,
  pub revision: Option<OpValsInt64>,
}
