use log::error;
use tokio::task::JoinError;

#[inline]
pub fn handle_join_error<T, E>(ret: Result<Result<T, E>, JoinError>, task_name: &str)
where
  E: core::fmt::Display,
{
  match ret {
    Ok(ret) => {
      if let Err(err) = ret {
        error!("Asynchronous task '{}' error: {}", task_name, err);
      }
    }
    Err(err) => error!("Asynchronous task '{}' Join error: {}", task_name, err),
  }
}

pub async fn wait_exit_signals() {
  // 同时监听 ctrl_c 和 kill 信号（SIGTERM）
  #[cfg(unix)]
  {
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("Failed to get SIGTERM signal handle");
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::select! {
      _ = ctrl_c => {
        log::info!("Received Ctrl+C signal, preparing to shutdown...");
      }
      _ = sigterm.recv() => {
        log::info!("Received kill(SIGTERM) signal, preparing to shutdown...");
      }
    }
  }
  #[cfg(not(unix))]
  {
    let ctrl_c = tokio::signal::ctrl_c();
    ctrl_c.await.unwrap();
  }
}
