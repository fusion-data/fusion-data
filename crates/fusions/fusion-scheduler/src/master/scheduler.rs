use std::time::Duration;

use chrono::Utc;
use fusion_server::{app::AppState, ctx::CtxW};
use hierarchical_hash_wheel_timer::{ClosureTimer, TimerReturn};
use modql::filter::OpValInt64;
use tokio::sync::mpsc;
use tracing::error;
use ultimate::Result;
use uuid::Uuid;

use crate::service::{
  sched_namespace::{SchedNamespaceFilter, SchedNamespaceSvc},
  sched_node::{NodeKind, SchedNode, SchedNodeFilter, SchedNodeForCreate, SchedNodeSvc},
  trigger_definition::TriggerDefinitionSvc,
};

use super::{CmdRunner, DbCmd, SchedulerConfig, TimerRef};

pub async fn loop_scheduler(
  app: AppState,
  scheduler_config: SchedulerConfig,
  timer_ref: TimerRef,
  db_tx: mpsc::Sender<DbCmd>,
  db_rx: mpsc::Receiver<DbCmd>,
) -> Result<Scheduler> {
  register(&scheduler_config, app.create_super_admin_ctx()).await?;

  let cmd_runner_handle = tokio::spawn(CmdRunner::new(app.clone(), db_rx).run());

  let mut scheduler = Scheduler { app, scheduler_config, timer_ref, db_tx };
  scheduler.init().await;

  cmd_runner_handle.await?;
  Ok(scheduler)
}

pub struct Scheduler {
  app: AppState,
  scheduler_config: SchedulerConfig,
  timer_ref: TimerRef,
  db_tx: mpsc::Sender<DbCmd>,
}

impl Scheduler {
  pub async fn init(&mut self) {
    start_heartbeat(&mut self.timer_ref, self.db_tx.clone(), &self.scheduler_config);

    loop {
      match self.scan_triggers().await {
        Ok(_) => (),
        Err(_) => error!("Failed to scan triggers"),
      };

      match self.scan_tasks().await {
        Ok(_) => (),
        Err(_) => error!("Failed to scan tasks"),
      };

      tokio::time::sleep(Duration::from_secs(30)).await;
    }
  }

  // 扫描触发器，计算下一次待执行任务并存储到数据库中
  async fn scan_triggers(&mut self) -> Result<()> {
    let node_id = self.scheduler_config.node_id();
    let ctx = self.app.create_super_admin_ctx();

    let namespaces = SchedNamespaceSvc::find_many(
      &ctx,
      vec![SchedNamespaceFilter { node_id: Some(OpValInt64::Eq(node_id).into()), ..Default::default() }],
    )
    .await?;
    let namespace_ids = namespaces.into_iter().map(|n| n.id).collect();

    TriggerDefinitionSvc::scan_next_triggers(&ctx, namespace_ids).await?;
    Ok(())
  }

  // 扫描任务，并添加任务到 timer_ref
  async fn scan_tasks(&mut self) -> Result<()> {
    Ok(())
  }
}

/// 注册节点。当节点ID不存在则插入。若节点已存在，先判断节点是否存活，若存活则返回错误，不存活则更新。
async fn register(scheduler_config: &SchedulerConfig, ctx: CtxW) -> Result<SchedNode> {
  let node_id = scheduler_config.node_id();
  let node_kind = NodeKind::Scheduler;
  let node_addr = scheduler_config.advertised_addr().to_string();

  let ctx = ctx.into_tx_mm_ctx();
  ctx.mm().dbx().begin_txn().await?;

  let sched_node = match SchedNodeSvc::find(
    &ctx,
    vec![SchedNodeFilter { id: Some(OpValInt64::Eq(node_id).into()), ..Default::default() }],
  )
  .await?
  {
    Some(sched_node) => sched_node,
    None => {
      let entity_c = SchedNodeForCreate {
        id: node_id.to_string(),
        kind: node_kind,
        addr: node_addr,
        status: Some(100),
        last_check_time: Some(Utc::now()),
      };
      SchedNodeSvc::create(&ctx, entity_c).await?;
      SchedNodeSvc::find_by_id(&ctx, node_id).await?
    }
  };

  ctx.mm().dbx().commit_txn().await?;

  Ok(sched_node)
}

async fn is_alive_node(node_addr: &str) -> bool {
  todo!()
}

/// 启动 node 心跳定时任务
fn start_heartbeat(timer_ref: &mut TimerRef, tx: mpsc::Sender<DbCmd>, conf: &SchedulerConfig) {
  let node_id = conf.node_id();
  let period = conf.heartbeat_interval();
  timer_ref.schedule_action_periodic(Uuid::nil(), Duration::from_secs(17), *period, move |_| {
    match tx.blocking_send(DbCmd::Heartbeat(node_id)) {
      Ok(_) => {}
      Err(e) => error!("Failed to send heartbeat to cmd runner: {}", e),
    };
    TimerReturn::Reschedule(())
  });
}
