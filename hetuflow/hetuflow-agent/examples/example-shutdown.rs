use std::time::{Duration, Instant};

use fusion_core::DataError;

#[tokio::main]
async fn main() -> Result<(), DataError> {
  let (tx, rx) = mea::shutdown::new_pair();
  for i in 0..3 {
    let rx = rx.clone();
    tokio::spawn(async move {
      println!("Task {} starting", i);
      rx.is_shutdown().await;
      println!("Task {} done", i);
    });
  }
  drop(rx);

  tokio::time::sleep(Duration::from_secs(2)).await;
  let inst = Instant::now();
  println!("Beginning shutdown: {:?}", inst);
  tx.shutdown();
  tx.await_shutdown().await;
  println!("Shutdown completed: {:?}", inst.elapsed());

  Ok(())
}
