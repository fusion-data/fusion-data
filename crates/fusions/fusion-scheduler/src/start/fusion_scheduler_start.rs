use fusion_server::app::get_app_state;

use crate::{endpoint::grpc_serve, master::Scheduler};

pub async fn fusion_scheduler_start() -> ultimate::Result<()> {
  let app = get_app_state();

  let (grpc_local_addr, grpc_serve) = grpc_serve(app).await?;

  let scheduler = Scheduler::new(app.clone(), grpc_local_addr)?;
  scheduler.init().await?;

  grpc_serve.await?;

  Ok(())
}
