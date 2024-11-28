use fusiondata_context::ctx::CtxW;
use tokio::sync::mpsc;
use tracing::{error, info};
use ultimate::application::Application;

use crate::service::{sched_namespace::SchedNamespace, sched_node::SchedNodeSvc};

use super::SchedCmd;

pub struct CmdRunner {
  app: Application,
  rx: mpsc::Receiver<SchedCmd>,
  sched_node_svc: SchedNodeSvc,
}

impl CmdRunner {
  pub fn new(app: Application, rx: mpsc::Receiver<SchedCmd>) -> Self {
    let sched_node_svc = app.component();
    Self { app, rx, sched_node_svc }
  }

  pub async fn run(mut self) {
    while let Some(msg) = self.rx.recv().await {
      match msg {
        SchedCmd::Heartbeat(node_id) => self.heartbeat(&node_id).await,
        SchedCmd::ListenNamespaces(sn) => self.compute_process_tasks(sn).await,
        SchedCmd::UnlistenNamespaces(_) => todo!(),
        SchedCmd::Stop => {
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
    let ctx = CtxW::new_with_app(self.app.clone());
    match self.sched_node_svc.heartbeat(&ctx, node_id).await {
      Ok(_) => {}
      Err(e) => error!("Failed to heartbeat to scheduler: {}", e),
    };
  }
}
