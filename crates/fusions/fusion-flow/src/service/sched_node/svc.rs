use std::time::Duration;

use chrono::Utc;
use fusion_flow_api::v1::sched_node::{NodeKind, NodeStatus};
use fusiondata_context::ctx::CtxW;
use modelsql::filter::{OpValInt32, OpValValue};
use ultimate_core::{Result, component::Component};

use crate::core::config::SchedulerConfig;

use super::{SchedNode, SchedNodeFilter, SchedNodeForCreate, SchedNodeForUpdate, bmc::SchedNodeBmc};

#[derive(Clone, Component)]
pub struct SchedNodeSvc {
  #[config]
  scheduler_config: SchedulerConfig,
}

impl SchedNodeSvc {
  pub async fn check_healthy_schedulers(&self, ctx: &CtxW, healthy_timeout: Duration) -> Result<Vec<SchedNode>> {
    let last_check_time = Utc::now() - healthy_timeout;
    self
      .update(
        ctx,
        vec![SchedNodeFilter {
          last_check_time: Some(OpValValue::Lt(serde_json::to_value(last_check_time).unwrap()).into()),
          status: Some(OpValInt32::Eq(NodeStatus::Healthy as i32).into()),
          ..Default::default()
        }],
        SchedNodeForUpdate { status: Some(NodeStatus::Unhealthy.into()), ..Default::default() },
      )
      .await?;

    self.find_many(ctx, vec![SchedNodeFilter { ..Default::default() }]).await
  }

  pub async fn find_by_id(&self, ctx: &CtxW, id: &str) -> Result<SchedNode> {
    SchedNodeBmc::find_by_id(ctx.mm(), id).await.map_err(Into::into)
  }

  pub async fn create(&self, ctx: &CtxW, entity_c: SchedNodeForCreate) -> Result<()> {
    SchedNodeBmc::insert(ctx.mm(), entity_c).await.map_err(Into::into)
  }

  pub async fn update_by_id(&self, ctx: &CtxW, id: &str, entity_u: SchedNodeForUpdate) -> Result<()> {
    SchedNodeBmc::update_by_id(ctx.mm(), id, entity_u).await.map_err(Into::into)
  }

  pub async fn find(&self, ctx: &CtxW, filter: Vec<SchedNodeFilter>) -> Result<Option<SchedNode>> {
    SchedNodeBmc::find_unique(ctx.mm(), filter).await.map_err(Into::into)
  }

  pub async fn find_many(&self, ctx: &CtxW, filter: Vec<SchedNodeFilter>) -> Result<Vec<SchedNode>> {
    SchedNodeBmc::find_many(ctx.mm(), filter, None).await.map_err(Into::into)
  }

  /// 注册节点。当节点ID不存在则插入。若节点已存在，先判断节点是否存活，若存活则返回错误，不存活则更新。
  pub async fn register(&self, ctx: &CtxW) -> Result<()> {
    let node_id = self.scheduler_config.node_id();
    let node_kind = NodeKind::Scheduler;
    let node_addr = self.scheduler_config.advertised_addr().to_string();
    let entity_c =
      SchedNodeForCreate { id: node_id, kind: node_kind, addr: node_addr, status: 100, last_check_time: Utc::now() };
    SchedNodeBmc::register(ctx.mm(), entity_c).await.map_err(Into::into)
  }

  /// 获取活跃主节点
  ///
  /// # Params
  /// alive_timeout: 主节点存活时间，超过此时间的主节点将被认为不在线，需要重新选举
  pub async fn find_active_master(&self, ctx: &CtxW, alive_timeout: Duration) -> Result<Option<SchedNode>> {
    SchedNodeBmc::find_active_master(ctx.mm(), Utc::now() - alive_timeout).await.map_err(Into::into)
  }

  /// 更新节点心跳
  pub async fn heartbeat(&self, ctx: &CtxW, node_id: &str) -> Result<()> {
    SchedNodeBmc::update_by_id(ctx.mm(), node_id, update_node_with_heartbeat())
      .await
      .map_err(Into::into)
  }

  /// 更新节点心跳并返回节点信息
  pub async fn heartbeat_and_return(&self, ctx: &CtxW, node_id: &str) -> Result<SchedNode> {
    SchedNodeBmc::update_and_return(ctx.mm(), node_id, update_node_with_heartbeat())
      .await
      .map_err(Into::into)
  }

  pub async fn update(&self, ctx: &CtxW, vec: Vec<SchedNodeFilter>, entity_u: SchedNodeForUpdate) -> Result<u64> {
    SchedNodeBmc::update(ctx.mm(), vec, entity_u).await.map_err(Into::into)
  }
}

fn update_node_with_heartbeat() -> SchedNodeForUpdate {
  SchedNodeForUpdate { status: Some(100), last_check_time: Some(Utc::now()), ..Default::default() }
}
