use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  // Step 1: Create a new CancellationToken
  let token = CancellationToken::new();

  // Step 2: Clone the token for use in another task
  let cloned_token = token.clone();

  // Task 1 - Wait for token cancellation or a long time
  let task1_handle = tokio::spawn(async move {
    tokio::select! {
        // Step 3: Using cloned token to listen to cancellation requests
        _ = cloned_token.cancelled() => {
            // The token was cancelled, task can shut down
            info!("The token was cancelled, task can shut down");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(9999)) => {
            // Long work has completed
            info!("Long work has completed")
        }
    }
  });

  // Task 2 - Cancel the original token after a small delay
  tokio::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Step 4: Cancel the original or cloned token to notify other tasks about shutting down gracefully
    token.cancel();
  });

  // Wait for tasks to complete
  task1_handle.await.unwrap()
}
