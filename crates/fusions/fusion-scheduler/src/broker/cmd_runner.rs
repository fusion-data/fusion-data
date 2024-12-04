use fusiondata_context::ctx::CtxW;
use tokio::sync::mpsc;
use tracing::{error, info};
use ultimate::application::Application;
use ultimate_db::ModelManager;

use crate::service::{sched_namespace::SchedNamespace, sched_node::SchedNodeSvc};

use super::SchedCmd;

pub struct CmdRunner {
  mm: ModelManager,
  sched_node_svc: SchedNodeSvc,
  rx: mpsc::Receiver<SchedCmd>,
}

impl CmdRunner {
  pub fn new(app: Application, rx: mpsc::Receiver<SchedCmd>) -> Self {
    let mm = app.component();
    let sched_node_svc = app.component();
    Self { mm, sched_node_svc, rx }
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
    let ctx = CtxW::new_super_admin(self.mm.clone());
    match self.sched_node_svc.heartbeat(&ctx, node_id).await {
      Ok(_) => {}
      Err(e) => error!("Failed to heartbeat to scheduler: {}", e),
    };
  }
}
