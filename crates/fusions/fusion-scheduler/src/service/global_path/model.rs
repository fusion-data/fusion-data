use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt64, OpValsString},
};
use sqlx::prelude::FromRow;
use ultimate_db::DbRowType;

#[derive(Debug, FromRow, Fields)]
pub struct GlobalPath {
  pub path: String,
  pub value: Option<String>,
  pub revision: i64,
}
impl DbRowType for GlobalPath {}

impl GlobalPath {
  pub const MASTER: &str = "/master";
  pub const NODE_TRIGGER: &str = "/node/trigger";
}

#[derive(Default, FilterNodes)]
pub struct GlobalPathFilter {
  pub path: Option<OpValsString>,
  pub revision: Option<OpValsInt64>,
}
