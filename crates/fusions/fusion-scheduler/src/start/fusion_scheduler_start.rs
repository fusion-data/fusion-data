use ultimate::{application::Application, timer::TimerPlugin, utils::handle_join_error};
use ultimate_db::DbPlugin;
use ultimate_grpc::GrpcPlugin;

use crate::{broker::spawn_loop, endpoint::grpc_serve};

pub async fn fusion_scheduler_start() -> ultimate::Result<()> {
  Application::builder().add_plugin(TimerPlugin).add_plugin(DbPlugin).add_plugin(GrpcPlugin).run().await?;

  let app: Application = Application::global();

  let (_rx, grpc_serve_fut) = grpc_serve(&app).await?;
  let grpc_serve_handle = tokio::spawn(grpc_serve_fut);

  let (master_handle, scheduler_handle) = spawn_loop();

  let (master_ret, scheduler_ret, grpc_ret) = tokio::join!(master_handle, scheduler_handle, grpc_serve_handle);

  handle_join_error(scheduler_ret, "scheduler");
  handle_join_error(master_ret, "master");
  handle_join_error(grpc_ret, "grpc serve");
  Ok(())
}
