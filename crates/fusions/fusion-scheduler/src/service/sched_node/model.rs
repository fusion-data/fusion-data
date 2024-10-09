use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsString},
};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::DbRowType;

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct SchedNode {
  pub id: String,
  pub kind: NodeKind,
  pub addr: String,
  pub status: i32,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedNode {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[repr(i32)]
pub enum NodeKind {
  Scheduler = 1,
  Worker = 2,
}

impl From<NodeKind> for sea_query::Value {
  fn from(value: NodeKind) -> Self {
    sea_query::Value::Int(Some(value as i32))
  }
}

impl sea_query::Nullable for NodeKind {
  fn null() -> sea_query::Value {
    sea_query::Value::Int(None)
  }
}

#[derive(Fields)]
pub struct SchedNodeForCreate {
  pub id: String,
  pub kind: NodeKind,
  pub addr: String,
}

#[derive(Default, Fields)]
pub struct SchedNodeForUpdate {
  pub kind: Option<NodeKind>,
  pub addr: Option<String>,
  pub status: Option<i32>,
}

#[derive(Default, FilterNodes)]
pub struct SchedNodeFilter {
  pub id: Option<OpValsString>,
  pub kind: Option<OpValsInt32>,
  pub status: Option<OpValsInt32>,
  pub addr: Option<OpValsString>,
}
