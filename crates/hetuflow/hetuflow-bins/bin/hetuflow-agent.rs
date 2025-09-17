use fusion_core::{DataError, utils::wait_exit_signals};

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
  wait_exit_signals().await;
  app.shutdown().await
}
