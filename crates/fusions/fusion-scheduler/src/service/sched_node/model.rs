use fusion_scheduler_api::v1::sched_node::{NodeKind, NodeStatus};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsInt64, OpValsString, OpValsValue},
};
use ultimate_db::{datetime_to_sea_value, DbRowType};

#[derive(Debug, Clone, FromRow, Fields)]
#[enum_def]
pub struct SchedNode {
  pub id: i64,
  pub kind: NodeKind,
  pub addr: String,
  pub status: NodeStatus,
  pub unhealth_count: i32,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedNode {}

#[derive(Fields)]
pub struct SchedNodeForCreate {
  pub id: String,
  pub kind: NodeKind,
  pub addr: String,
  pub status: Option<i32>,
  pub last_check_time: Option<UtcDateTime>,
}

#[derive(Default, Fields)]
pub struct SchedNodeForUpdate {
  pub kind: Option<NodeKind>,
  pub addr: Option<String>,
  pub status: Option<i32>,
  pub last_check_time: Option<UtcDateTime>,
}

#[derive(Default, FilterNodes)]
pub struct SchedNodeFilter {
  pub id: Option<OpValsInt64>,
  pub kind: Option<OpValsInt32>,
  pub status: Option<OpValsInt32>,
  pub addr: Option<OpValsString>,
  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub last_check_time: Option<OpValsValue>,
}
