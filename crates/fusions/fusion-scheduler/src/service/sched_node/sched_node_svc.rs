use std::time::Duration;

use chrono::Utc;
use fusion_scheduler_api::v1::sched_node::NodeStatus;
use fusiondata_context::ctx::CtxW;
use ultimate::Result;
use ultimate_db::modql::filter::{OpValInt32, OpValValue};

use super::{sched_node_bmc::SchedNodeBmc, SchedNode, SchedNodeFilter, SchedNodeForCreate, SchedNodeForUpdate};

pub struct SchedNodeSvc;
impl SchedNodeSvc {
  pub async fn check_healthy_schedulers(ctx: &CtxW, healthy_timeout: Duration) -> Result<Vec<SchedNode>> {
    let last_check_time = Utc::now() - healthy_timeout;
    Self::update(
      &ctx,
      vec![SchedNodeFilter {
        last_check_time: Some(OpValValue::Lt(serde_json::to_value(last_check_time).unwrap()).into()),
        status: Some(OpValInt32::Eq(NodeStatus::Healthy as i32).into()),
        ..Default::default()
      }],
      SchedNodeForUpdate { status: Some(NodeStatus::Unhealthy.into()), ..Default::default() },
    )
    .await?;

    Self::find_many(&ctx, vec![SchedNodeFilter { ..Default::default() }]).await
  }

  pub async fn find_by_id(ctx: &CtxW, id: i64) -> Result<SchedNode> {
    SchedNodeBmc::find_by_id(ctx.mm(), id).await.map_err(Into::into)
  }

  pub async fn create(ctx: &CtxW, entity_c: SchedNodeForCreate) -> Result<()> {
    SchedNodeBmc::insert(ctx.mm(), entity_c).await.map_err(Into::into)
  }

  pub async fn update_by_id(ctx: &CtxW, id: i64, entity_u: SchedNodeForUpdate) -> Result<()> {
    SchedNodeBmc::update_by_id(ctx.mm(), id, entity_u).await.map_err(Into::into)
  }

  pub async fn find(ctx: &CtxW, filter: Vec<SchedNodeFilter>) -> Result<Option<SchedNode>> {
    SchedNodeBmc::find_unique(ctx.mm(), filter).await.map_err(Into::into)
  }

  pub async fn find_many(ctx: &CtxW, filter: Vec<SchedNodeFilter>) -> Result<Vec<SchedNode>> {
    SchedNodeBmc::find_many(ctx.mm(), filter, None).await.map_err(Into::into)
  }

  /// 获取活跃主节点
  pub async fn find_active_master(ctx: &CtxW, alive_timeout: Duration) -> Result<Option<SchedNode>> {
    let valid_check_time = Utc::now() - alive_timeout;
    SchedNodeBmc::find_active_master(ctx.mm(), valid_check_time).await.map_err(Into::into)
  }

  pub async fn heartbeat(ctx: &CtxW, node_id: i64) -> Result<()> {
    let entity_u = SchedNodeForUpdate { status: Some(100), last_check_time: Some(Utc::now()), ..Default::default() };
    SchedNodeBmc::update_by_id(ctx.mm(), node_id, entity_u).await.map_err(Into::into)
  }

  pub async fn heartbeat_and_return(ctx: &CtxW, node_id: i64) -> Result<SchedNode> {
    let entity_u = SchedNodeForUpdate { status: Some(100), last_check_time: Some(Utc::now()), ..Default::default() };
    SchedNodeBmc::update_and_return(ctx.mm(), node_id, entity_u).await.map_err(Into::into)
  }

  pub async fn update(ctx: &CtxW, vec: Vec<SchedNodeFilter>, entity_u: SchedNodeForUpdate) -> Result<u64> {
    SchedNodeBmc::update(ctx.mm(), vec, entity_u).await.map_err(Into::into)
  }
}
