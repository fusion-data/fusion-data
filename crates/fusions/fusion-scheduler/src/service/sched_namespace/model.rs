use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsInt64},
};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::DbRowType;

/// 调度命名空间。
#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct SchedNamespace {
  pub id: i32,
  pub tenant_id: i32,
  pub namespace: String,
  pub node_id: String,

  /// 节点最后活跃时间。当节点最后活跃时间小于配置的节点超时时间，则认为节点已离线。
  /// 节点离线后，其它节点可以更新 node_id 以绑定与 namespace 的关联关系
  pub node_last_time: UtcDateTime,

  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedNamespace {}

pub struct AssociateNamespaceWithScheduler {
  pub namespace: String,
  pub scheduler_id: String,
}

#[derive(Default, FilterNodes)]
pub struct SchedNamespaceFilter {
  pub node_id: Option<OpValsInt64>,
  pub tenant_id: Option<OpValsInt32>,
}
