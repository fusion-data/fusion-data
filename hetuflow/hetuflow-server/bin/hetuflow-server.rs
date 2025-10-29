use fusion_core::{DataError, utils::wait_exit_signals};

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
  wait_exit_signals().await;
  app.shutdown_and_await().await
}
