use std::time::{Duration, Instant};

use tokio::time::{interval, pause, sleep, timeout};
use tracing::info;

/// Pauses the virtual timer, and prints the elapsed time is 0ms.
#[tokio::test]
async fn paused_time() {
  // Pause virtual timer
  pause();

  let start = Instant::now();
  sleep(Duration::from_millis(500)).await;
  println!("{:?}ms", start.elapsed().as_millis());
}

#[tokio::test(start_paused = true)]
async fn paused_time_use_macro() {
  let start = Instant::now();
  sleep(Duration::from_millis(500)).await;
  println!("{:?}ms", start.elapsed().as_millis());
}

#[tokio::test(start_paused = true)]
async fn interval_with_paused_time() {
  tracing_subscriber::fmt::init();
  let mut interval = interval(Duration::from_millis(300));
  info!("Starting interval");
  let _ = timeout(Duration::from_secs(1), async move {
    loop {
      interval.tick().await;
      info!("Tick!");
    }
  })
  .await;
}
