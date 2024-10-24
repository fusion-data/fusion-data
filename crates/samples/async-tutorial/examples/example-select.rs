use std::{thread::sleep, time::Duration};

use tokio::sync::oneshot;
async fn some_operation() -> String {
  // sleep(Duration::from_millis(1));
  "Compute value here".to_string()
}

#[tokio::main]
async fn main() {
  let (mut tx1, rx1) = oneshot::channel();
  let (tx2, rx2) = oneshot::channel();

  tokio::spawn(async {
    // Select on the operation and the oneshot's
    // `closed()` notification.
    tokio::select! {
        _ = tx1.closed() => {
            // `some_operation()` is canceled, the
            // task completes and `tx1` is dropped.
        }
        val = some_operation() => {
            let _ = tx1.send(val);
        }
    }
  });

  tokio::spawn(async {
    let _ = tx2.send("two");
  });

  tokio::select! {
      val = rx1 => {
          println!("rx1 completed first with {:?}", val);
      }
      val = rx2 => {
          println!("rx2 completed first with {:?}", val);
      }
  }
}
