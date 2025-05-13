//! 调度员
//!
use fusiondata_context::ctx::CtxW;
use tokio::{sync::mpsc, task::JoinHandle};
use ultimate_core::{application::Application, component::Component, timer::Timer};

mod cmd_runner;
mod master;
mod model;
mod scheduler;
pub mod start;

use cmd_runner::CmdRunner;
pub use master::*;
pub use model::*;
pub use scheduler::*;

use fusion_flow::service::sched_node::SchedNodeSvc;

#[derive(Clone, Component)]
pub struct Broker {
  #[component]
  sched_node_svc: SchedNodeSvc,

  #[component]
  timer: Timer,
}

impl Broker {
  pub async fn spawn_loop(
    &self,
  ) -> ultimate_core::Result<(JoinHandle<ultimate_core::Result<()>>, JoinHandle<ultimate_core::Result<Scheduler>>)> {
    self.sched_node_svc.register(&CtxW::new_super_admin(Application::global().component())).await?;

    let master_handle = {
      let f = loop_master(Application::global());
      tokio::spawn(f)
    };

    let scheduler_handle = {
      let (db_tx, db_rx) = mpsc::channel(1024);
      let f = loop_scheduler(Application::global(), self.timer.timer_ref(), db_tx, db_rx);
      tokio::spawn(f)
    };

    Ok((master_handle, scheduler_handle))
  }
}
