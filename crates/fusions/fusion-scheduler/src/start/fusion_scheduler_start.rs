use hierarchical_hash_wheel_timer::thread_timer::TimerWithThread;
use tracing::error;
use ultimate::{application::Application, utils::handle_join_error};
use ultimate_db::DbPlugin;

use crate::{broker::spawn_loop, endpoint::grpc_serve};

pub async fn fusion_scheduler_start() -> ultimate::Result<()> {
  Application::builder().add_plugin(DbPlugin).run().await;

  let app = Application::global();

  let (rx, grpc_serve_fut) = grpc_serve(&app).await?;
  let grpc_serve_handle = tokio::spawn(grpc_serve_fut);
  let grpc_start_info = rx.await?;

  let timer_core = TimerWithThread::for_uuid_closures();

  let (master_handle, scheduler_handle) = spawn_loop(app.clone(), grpc_start_info.local_addr, timer_core.timer_ref());

  let (master_ret, scheduler_ret, grpc_ret) = tokio::join!(master_handle, scheduler_handle, grpc_serve_handle);
  match timer_core.shutdown() {
    Ok(_) => (),
    Err(e) => error!("The hash_wheel_timer shutdown failed: {:?}", e),
  };

  handle_join_error(scheduler_ret, "scheduler");
  handle_join_error(master_ret, "master");
  handle_join_error(grpc_ret, "grpc serve");
  Ok(())
}
