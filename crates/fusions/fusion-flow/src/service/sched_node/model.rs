use fusion_flow_api::v1::sched_node::{NodeKind, NodeStatus};
use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsString, OpValsValue},
  postgres::PgRowType,
  utils::datetime_to_sea_value,
};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;

#[derive(Debug, Clone, FromRow, Fields)]
#[enum_def]
pub struct SchedNode {
  pub id: String,
  pub kind: NodeKind,
  pub addr: String,
  pub status: NodeStatus,
  pub unhealth_count: i32,
  pub last_check_time: Option<UtcDateTime>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl PgRowType for SchedNode {}

#[derive(Fields)]
pub struct SchedNodeForCreate {
  pub id: String,
  pub kind: NodeKind,
  pub addr: String,
  pub status: i32,
  pub last_check_time: UtcDateTime,
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
  pub id: Option<OpValsString>,
  pub kind: Option<OpValsInt32>,
  pub status: Option<OpValsInt32>,
  pub addr: Option<OpValsString>,
  #[modelsql(to_sea_value_fn = "datetime_to_sea_value")]
  pub last_check_time: Option<OpValsValue>,
}
