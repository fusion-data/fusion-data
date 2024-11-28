//! 调度员
//!
use fusiondata_context::ctx::CtxW;
use tokio::{sync::mpsc, task::JoinHandle};
use ultimate::{application::Application, component::Component, timer::Timer};

mod cmd_runner;
mod config;
mod master;
mod model;
mod scheduler;

use cmd_runner::CmdRunner;
pub use config::*;
pub use master::*;
pub use model::*;
pub use scheduler::*;

use crate::service::sched_node::SchedNodeSvc;

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
  ) -> ultimate::Result<(JoinHandle<ultimate::Result<()>>, JoinHandle<ultimate::Result<Scheduler>>)> {
    let (db_tx, db_rx) = mpsc::channel(1024);

    self.sched_node_svc.register(&CtxW::new_with_app(Application::global())).await?;

    let master_handle = {
      let f = loop_master(Application::global());
      tokio::spawn(f)
    };

    let scheduler_handle = {
      let f = loop_scheduler(Application::global(), self.timer.timer_ref(), db_tx, db_rx);
      tokio::spawn(f)
    };

    Ok((master_handle, scheduler_handle))
  }
}
