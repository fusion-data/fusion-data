//! 调度员
//!
use std::net::SocketAddr;

use fusion_server::app::AppState;
use hierarchical_hash_wheel_timer::{
  thread_timer::{self, TimerWithThread},
  OneShotClosureState, PeriodicClosureState,
};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::service::sched_node::SchedNode;

mod cmd_runner;
mod config;
mod master;
mod model;
mod scheduler;

use cmd_runner::CmdRunner;
use config::*;
use master::*;
pub use model::*;
use scheduler::*;

// pub type TimerCore = TimerWithThread<uuid::Uuid, OneShotClosureState<uuid::Uuid>, PeriodicClosureState<uuid::Uuid>>;
pub type TimerRef =
  thread_timer::TimerRef<uuid::Uuid, OneShotClosureState<uuid::Uuid>, PeriodicClosureState<uuid::Uuid>>;

enum MasterSchedulers {
  Master(SchedNode),
  Schedulers(Vec<SchedNode>),
}

pub fn spawn_loop(
  app: AppState,
  grpc_sock_addr: SocketAddr,
  timer_ref: TimerRef,
) -> (JoinHandle<ultimate::Result<()>>, JoinHandle<ultimate::Result<Scheduler>>) {
  let scheduler_config =
    match SchedulerConfig::try_new(app.configuration_state().underling(), grpc_sock_addr.to_string()) {
      Ok(c) => c,
      Err(e) => panic!("Parse scheduler config failed: {}", e),
    };
  let (db_tx, db_rx) = mpsc::channel(1024);

  let master_handle = {
    let f = loop_master(app.clone(), scheduler_config.clone(), timer_ref.clone(), db_tx.clone());
    tokio::spawn(f)
  };

  let scheduler_handle = {
    let f = loop_scheduler(app, scheduler_config, timer_ref, db_tx, db_rx);
    tokio::spawn(f)
  };

  (master_handle, scheduler_handle)
}
