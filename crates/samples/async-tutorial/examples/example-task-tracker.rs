use std::time::Duration;

use tokio::time::sleep;
use tokio_util::task::TaskTracker;
use tracing::info;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let tracker = TaskTracker::new();

  for i in 0..10 {
    tracker.spawn(some_operation(i));
  }

  // Once we spawned everything, we close the tracker.
  tracker.close();

  // Wait for everythong to finish.
  tracker.wait().await;

  info!("This is printed after all of the tasks.");
}

async fn some_operation(i: u64) {
  sleep(Duration::from_millis(100 * i)).await;
  info!("Task {} shutting down.", i);
}
