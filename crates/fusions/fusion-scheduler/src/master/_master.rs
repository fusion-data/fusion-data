use std::time::Duration;

use fusion_server::{app::AppState, ctx::CtxW};
use tokio::sync::mpsc;
use ultimate::Result;

use crate::service::{
  global_path::{GlobalPath, GlobalPathSvc},
  sched_node::{SchedNode, SchedNodeSvc},
};

use super::{DbCmd, SchedulerConfig, TimerRef};

pub async fn loop_master(
  app: AppState,
  scheduler_config: SchedulerConfig,
  timer_ref: TimerRef,
  db_tx: mpsc::Sender<DbCmd>,
) -> Result<()> {
  let mut master_opt: Option<SchedNode> = None;
  while master_opt.is_none() {
    let alive_timeout = *scheduler_config.alive_timeout();
    let ctx = app.create_super_admin_ctx();
    let node_id = scheduler_config.node_id();
    let node = match SchedNodeSvc::find_active_master(&ctx, alive_timeout).await? {
      Some(node) => node,
      None => bind_or_get_master(&ctx, node_id).await?,
    };
    if node.id == node_id {
      master_opt = Some(node);
    } else {
      tokio::time::sleep(Duration::from_secs(30)).await;
    }
  }

  let master = Master { app, scheduler_config, timer_ref, db_tx };
  master.run().await
}

pub struct Master {
  app: AppState,
  scheduler_config: SchedulerConfig,
  timer_ref: TimerRef,
  db_tx: mpsc::Sender<DbCmd>,
}

impl Master {
  pub async fn run(mut self) -> Result<()> {
    // 检查 scheduler 健康状态
    // 检查是否需要平衡 namespace
    // 处理新添加 namespace 关联哪个 scheduler

    Ok(())
  }
}

/// 尝试绑定并获取最新主节点
async fn bind_or_get_master(ctx: &CtxW, node_id: i64) -> Result<SchedNode> {
  // 最多尝试 5 次
  let mut count = 5;
  while count > 0 {
    let opt_path = GlobalPathSvc::find_unique(&ctx, GlobalPath::MASTER).await?;
    let ret = GlobalPathSvc::obtain_optimistic_lock(
      &ctx,
      GlobalPath::MASTER,
      opt_path.map(|p| p.revision).unwrap_or_default(),
      Some(node_id.to_string()),
    )
    .await?;

    if ret {
      let node = SchedNodeSvc::heartbeat_and_return(&ctx, node_id).await?;
      return Ok(node);
    } else {
      match SchedNodeSvc::find_active_master(&ctx, Duration::from_secs(10)).await? {
        Some(node) => return Ok(node),
        None => count -= 1,
      }
    }
  }

  panic!("Failed to bind & get active master");
}
