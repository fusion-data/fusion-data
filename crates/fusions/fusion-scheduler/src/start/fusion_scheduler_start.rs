use fusion_server::app::get_app_state;
use hierarchical_hash_wheel_timer::thread_timer::TimerWithThread;
use tokio::sync::oneshot;

use crate::{endpoint::grpc_serve, master::spawn_loop};

pub async fn fusion_scheduler_start() -> ultimate::Result<()> {
  let app = get_app_state();

  let (tx, rx) = oneshot::channel();
  let grpc_handle = tokio::spawn(grpc_serve(app, tx));

  let grpc_start_info = rx.await?;

  let timer_core = TimerWithThread::for_uuid_closures();

  let (master_handle, scheduler_handle) = spawn_loop(app.clone(), grpc_start_info.local_addr, &timer_core);

  let (master_ret, scheduler_ret, grpc_ret) = tokio::join!(master_handle, scheduler_handle, grpc_handle);
  timer_core.shutdown();
  scheduler_ret??;
  master_ret??;
  grpc_ret??;
  Ok(())
}
