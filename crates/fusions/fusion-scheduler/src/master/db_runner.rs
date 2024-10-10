use fusion_server::app::AppState;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{error, info};

use crate::service::{sched_namespace::SchedNamespace, sched_node::SchedNodeSvc};

use super::DbCmd;

pub fn loop_db_runner(runner: DbRunner) -> JoinHandle<()> {
  tokio::spawn(runner.run())
}

pub struct DbRunner {
  app_state: AppState,
  rx: mpsc::Receiver<DbCmd>,
}

impl DbRunner {
  pub fn new(app_state: AppState, rx: mpsc::Receiver<DbCmd>) -> Self {
    Self { app_state, rx }
  }

  async fn run(mut self) {
    while let Some(msg) = self.rx.recv().await {
      match msg {
        DbCmd::Heartbeat(node_id) => self.heartbeat(node_id).await,
        DbCmd::ListenNamespaces(sn) => self.compute_process_tasks(sn).await,
        DbCmd::UnlistenNamespaces(_) => todo!(),
        DbCmd::Stop => {
          self.rx.close();
          info!("Receive 'DbCmd::Stop', db_runner begin stop.");
        }
      }
    }
  }

  async fn compute_process_tasks(&self, sn: Vec<SchedNamespace>) {
    todo!()
  }

  async fn heartbeat(&self, node_id: &str) {
    let ctx = self.app_state.create_super_admin_ctx();
    match SchedNodeSvc::heartbeat(&ctx, node_id).await {
      Ok(_) => {}
      Err(e) => error!("Failed to heartbeat to scheduler: {}", e),
    };
  }
}
