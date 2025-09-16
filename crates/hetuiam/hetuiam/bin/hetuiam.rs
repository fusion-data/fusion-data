use fusion_core::{DataError, utils::wait_exit_signals};

use hetuiam::start::start_hetuiam;

#[tokio::main]
async fn main() -> Result<(), DataError> {
  start_hetuiam().await?;
  wait_exit_signals().await;
  Ok(())
}
