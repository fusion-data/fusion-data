use std::{net::SocketAddr, sync::Arc};

use fusion_server::{app::AppState, ctx::CtxW};
use hierarchical_hash_wheel_timer::{
  thread_timer::{self, TimerWithThread},
  OneShotClosureState, PeriodicClosureState,
};
use modql::filter::OpValString;
use tracing::error;
use ultimate::{DataError, Result};
use ultimate_db::{DbState, ModelManager};

use crate::service::{
  sched_namespace::SchedNamespaceSvc,
  sched_node::{NodeKind, SchedNode, SchedNodeFilter, SchedNodeForCreate, SchedNodeForUpdate, SchedNodeSvc},
};

use super::SchedulerConfig;

pub type TimerCore = TimerWithThread<uuid::Uuid, OneShotClosureState<uuid::Uuid>, PeriodicClosureState<uuid::Uuid>>;
type TimerRef = thread_timer::TimerRef<uuid::Uuid, OneShotClosureState<uuid::Uuid>, PeriodicClosureState<uuid::Uuid>>;

pub struct Scheduler {
  timer_core: TimerCore,
  app_state: AppState,
  scheduler_config: SchedulerConfig,
}

impl Scheduler {
  pub fn new(app_state: AppState, grpc_local_addr: SocketAddr) -> Result<Self> {
    let mut scheduler_config: SchedulerConfig = app_state.configuration_state().underling().get("fusion-scheduler")?;
    if scheduler_config.advertised_addr.is_none() {
      scheduler_config.advertised_addr = Some(grpc_local_addr.to_string());
    }
    Ok(Self { timer_core: TimerWithThread::for_uuid_closures(), app_state, scheduler_config })
  }

  pub async fn init(&self) -> Result<()> {
    let sched_node = self.register().await?;

    self.associate_namespaces(sched_node).await?;

    Ok(())
  }

  pub fn shutdown(self) {
    match self.timer_core.shutdown() {
      Ok(_) => {}
      Err(err) => error!("{:?}", err),
    };
  }

  /// 注册节点。当节点ID不存在则插入。若节点已存在，先判断节点是否存活，若存活则返回错误，不存活则更新。
  async fn register(&self) -> Result<SchedNode> {
    // TODO 从配置文件读取
    let node_id = "scheduler";
    let node_kind = NodeKind::Scheduler;
    let node_addr = "127.0.0.1:8080";

    let ctx = self.app_state.create_super_admin_ctx();
    let ctx = {
      let mm = ctx.mm().get_or_clone_with_txn()?;
      mm.dbx().begin_txn().await?;
      ctx.with_mm(mm)
    };

    match SchedNodeSvc::find(
      &ctx,
      vec![SchedNodeFilter { id: Some(OpValString::Eq(node_id.to_string()).into()), ..Default::default() }],
    )
    .await?
    {
      Some(sched_node) => {
        if self.is_alive_node(&sched_node.addr).await {
          return Err(DataError::confilicted(format!("Node '{}|{}' is in a alive state.", node_id, node_addr)));
        } else {
          // 节点已下线，更新记录
          let entity_u =
            SchedNodeForUpdate { kind: Some(node_kind), addr: Some(node_addr.to_string()), status: Some(100) };
          SchedNodeSvc::update_by_id(&ctx, node_id, entity_u).await?;
        }
      }
      None => {
        let entity_c = SchedNodeForCreate { id: node_id.to_string(), kind: node_kind, addr: node_id.to_string() };
        SchedNodeSvc::create(&ctx, entity_c).await?;
      }
    };

    let sched_node = SchedNodeSvc::find_by_id(&ctx, node_id).await?;

    ctx.mm().dbx().commit_txn().await?;

    Ok(sched_node)
  }

  async fn is_alive_node(&self, node_addr: &str) -> bool {
    todo!()
  }

  /// 关联调度命名空间
  async fn associate_namespaces(&self, sched_node: SchedNode) -> Result<()> {
    let ctx = self.app_state.create_super_admin_ctx();

    let datas = self.scheduler_config.namespaces.iter().map(|namespace| {}).collect();

    let associated_data = SchedNamespaceSvc::associate_scheduler(&ctx, datas).await?;

    todo!()
  }
}
