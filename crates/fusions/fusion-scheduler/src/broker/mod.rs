//! 调度员
//!
use tokio::{sync::mpsc, task::JoinHandle};
use ultimate::{application::Application, timer::Timer};

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

pub fn spawn_loop() -> (JoinHandle<ultimate::Result<()>>, JoinHandle<ultimate::Result<Scheduler>>) {
  let app = Application::global();

  let (db_tx, db_rx) = mpsc::channel(1024);

  let timer: Timer = app.component();

  let master_handle = {
    let f = loop_master(app.clone(), db_tx.clone());
    tokio::spawn(f)
  };

  let scheduler_handle = {
    let f = loop_scheduler(app, timer.timer_ref(), db_tx, db_rx);
    tokio::spawn(f)
  };

  (master_handle, scheduler_handle)
}
