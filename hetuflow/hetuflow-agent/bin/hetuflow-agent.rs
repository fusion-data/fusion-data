use fusion_core::DataError;
use log::info;

use hetuflow_agent::application::AgentApplication;

/// 启动 Hetuflow Agent
///
/// ```shell
/// cargo run --bin hetuflow-agent
/// ```
#[tokio::main]
async fn main() -> Result<(), DataError> {
  let app = AgentApplication::new().await?;
  app.start().await?;

  let ctrl_c = tokio::signal::ctrl_c();
  #[cfg(unix)]
  {
    // 同时监听 ctrl_c 和 kill 信号（SIGTERM）
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
      _ = ctrl_c => {
        info!("收到 Ctrl+C 信号，准备关闭...");
      }
      _ = sigterm.recv() => {
        info!("收到 kill(SIGTERM) 信号，准备关闭...");
      }
    }
  }
  #[cfg(not(unix))]
  {
    ctrl_c.await?;
  }

  app.shutdown().await
}
