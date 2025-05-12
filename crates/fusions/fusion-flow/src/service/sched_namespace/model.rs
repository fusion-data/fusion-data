use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsInt64, OpValsString},
  DbRowType,
};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;

/// 调度命名空间。
#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct SchedNamespace {
  pub id: i64,
  pub tenant_id: i32,
  pub namespace: String,
  pub node_id: Option<String>,
  pub status: i32,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedNamespace {}

#[derive(Default, FilterNodes)]
pub struct SchedNamespaceFilter {
  pub id: Option<OpValsInt64>,
  pub node_id: Option<OpValsString>,
  pub tenant_id: Option<OpValsInt32>,
  pub status: Option<OpValsInt32>,
}

#[derive(Default, Fields)]
pub struct SchedNamespaceForUpdate {
  pub node_id: Option<String>,
  pub status: Option<i32>,
}
