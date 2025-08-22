use log::info;
use tokio::select;
use tokio::signal::unix::{SignalKind, signal};
use ultimate_core::DataError;

use hetuflow_server::application::ServerApplication;

/// 启动 Hetuflow Server
///
/// ```bash
/// cargo run --bin hetuflow-server
/// ```
#[tokio::main]
async fn main() -> Result<(), DataError> {
  let app = ServerApplication::new().await?;
  app.start().await?;

  // 同时监听 ctrl_c 和 kill 信号（SIGTERM）
  let mut sigterm = signal(SignalKind::terminate())?;
  let ctrl_c = tokio::signal::ctrl_c();
  select! {
    _ = ctrl_c => {
      info!("收到 Ctrl+C 信号，准备关闭...");
    }
    _ = sigterm.recv() => {
      info!("收到 kill(SIGTERM) 信号，准备关闭...");
    }
  }

  app.shutdown().await
}
