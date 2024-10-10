use fusion_server::app::get_app_state;
use tokio::sync::oneshot;
use tracing::error;

use crate::{endpoint::grpc_serve, master::Scheduler};

pub async fn fusion_scheduler_start() -> ultimate::Result<()> {
  let app = get_app_state();

  let (grpc_start_tx, grpc_start_rx) = oneshot::channel();

  let grpc_join_handle = tokio::spawn(grpc_serve(app, grpc_start_tx));

  let grpc_start_info = grpc_start_rx.await?;

  let mut scheduler = Scheduler::new(app.clone(), grpc_start_info)?;
  scheduler.init().await?;

  match grpc_join_handle.await? {
    Ok(_) => {}
    Err(e) => error!("Start grpc server failed: {:?}", e),
  };
  scheduler.shutdown().await;

  Ok(())
}
