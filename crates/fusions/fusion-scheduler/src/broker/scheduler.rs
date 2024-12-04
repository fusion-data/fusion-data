use std::{sync::Arc, time::Duration};

use fusiondata_context::ctx::CtxW;
use tokio::sync::mpsc;
use tracing::error;
use ultimate::{
  application::Application,
  configuration::ConfigRegistry,
  timer::{TimerRef, TimerReturn},
  Result,
};
use uuid::Uuid;

use crate::service::trigger_definition::TriggerDefinitionSvc;

use super::{CmdRunner, SchedCmd, SchedulerConfig};

pub struct SchedCmdMpsc {
  pub(crate) tx: mpsc::Sender<SchedCmd>,
  pub(crate) rx: Arc<mpsc::Receiver<SchedCmd>>,
}
impl Default for SchedCmdMpsc {
  fn default() -> Self {
    let (tx, rx) = mpsc::channel(1024);
    SchedCmdMpsc { tx, rx: Arc::new(rx) }
  }
}

pub async fn loop_scheduler(
  app: Application,
  timer_ref: TimerRef,
  db_tx: mpsc::Sender<SchedCmd>,
  db_rx: mpsc::Receiver<SchedCmd>,
) -> Result<Scheduler> {
  let scheduler_config: SchedulerConfig = app.get_config()?;

  let cmd_runner_handle = tokio::spawn(CmdRunner::new(app.clone(), db_rx).run());

  let mut scheduler = Scheduler { app, scheduler_config, timer_ref, db_tx };
  scheduler.init().await;

  cmd_runner_handle.await?;
  Ok(scheduler)
}

pub struct Scheduler {
  app: Application,
  scheduler_config: SchedulerConfig,
  timer_ref: TimerRef,
  db_tx: mpsc::Sender<SchedCmd>,
}

impl Scheduler {
  pub async fn init(&mut self) {
    start_heartbeat(&mut self.timer_ref, self.db_tx.clone(), &self.scheduler_config);

    loop {
      match self.scan_triggers().await {
        Ok(_) => (),
        Err(e) => error!("Failed to scan triggers: {:?}", e),
      };

      match self.scan_tasks().await {
        Ok(_) => (),
        Err(e) => error!("Failed to scan tasks: {:?}", e),
      };

      tokio::time::sleep(Duration::from_secs(30)).await;
    }
  }

  // 扫描触发器，计算下一次待执行任务并存储到数据库中
  async fn scan_triggers(&mut self) -> Result<()> {
    let node_id = self.scheduler_config.node_id();
    let ctx = CtxW::new_super_admin(self.app.component());

    TriggerDefinitionSvc::scan_and_compute_next_triggers(&ctx, &node_id).await?;

    Ok(())
  }

  // 扫描任务，创建 TaskJob 并添加到 timer_ref
  async fn scan_tasks(&mut self) -> Result<()> {
    Ok(())
  }
}

/// 启动 scheduler node 心跳定时任务
fn start_heartbeat(timer_ref: &mut TimerRef, tx: mpsc::Sender<SchedCmd>, conf: &SchedulerConfig) {
  let node_id = conf.node_id();
  let period = conf.heartbeat_interval();
  timer_ref.schedule_action_periodic(Uuid::now_v7(), Duration::from_secs(17), *period, move |job_id| {
    match tx.blocking_send(SchedCmd::Heartbeat(node_id.clone())) {
      Ok(_) => {}
      Err(e) => error!("[job:{}] Failed to send heartbeat to cmd runner: {}", job_id, e),
    };
    TimerReturn::Reschedule(())
  });
}
