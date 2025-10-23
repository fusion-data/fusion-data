use fusions::core::{DataError, utils::wait_exit_signals};

use jieyuan_server::start::start_jieyuan;

#[tokio::main]
async fn main() -> Result<(), DataError> {
  start_jieyuan().await?;
  wait_exit_signals().await;
  Ok(())
}
