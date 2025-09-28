use fusion_core::{DataError, utils::wait_exit_signals};

use jieyuan::start::start_jieyuan;

#[tokio::main]
async fn main() -> Result<(), DataError> {
  start_jieyuan().await?;
  wait_exit_signals().await;
  Ok(())
}
