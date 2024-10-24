use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsInt64},
};
use ultimate_db::DbRowType;

/// 调度命名空间。
#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct SchedNamespace {
  pub id: i32,
  pub tenant_id: i32,
  pub namespace: String,
  pub node_id: Option<i64>,
  pub status: i32,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedNamespace {}

#[derive(Default, FilterNodes)]
pub struct SchedNamespaceFilter {
  pub id: Option<OpValsInt32>,
  pub node_id: Option<OpValsInt64>,
  pub tenant_id: Option<OpValsInt32>,
  pub status: Option<OpValsInt32>,
}

#[derive(Default, Fields)]
pub struct SchedNamespaceForUpdate {
  pub node_id: Option<i64>,
  pub status: Option<i32>,
}
