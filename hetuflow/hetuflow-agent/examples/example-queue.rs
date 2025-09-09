use std::{sync::Arc, time::Duration};

use futures_util::FutureExt;
use mea::mpsc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
  demo_mea_mpsc().await;
  demo_kanal_mpmc().await;
}

async fn demo_kanal_mpmc() {
  let (tx, rx) = kanal::bounded_async(2);
  let rx = Arc::new(rx);
  let (shutdown_tx, _) = broadcast::channel::<()>(1);

  let rx0 = rx.clone();
  let mut shutdown_rx = shutdown_tx.subscribe();
  tokio::spawn(async move {
    let rx0_fut = rx0.recv().fuse();
    let shutdown_fut = shutdown_rx.recv().fuse();
    futures_util::pin_mut!(rx0_fut, shutdown_fut);
    futures_util::select! {
      v = rx0_fut => {
        println!("rx0 recv: {:?}", v);
      }
      _ = shutdown_fut => {
        println!("rx0 recv shutdown signal");
      }
    }
  });

  let rx1 = rx.clone();
  tokio::spawn(async move {
    println!("rx1 started");
    loop {
      match rx1.recv().await {
        Ok(v) => {
          println!("rx1 recv: {:?}", v);
        }
        Err(e) => {
          println!("rx1 recv error: {:?}", e);
          break;
        }
      }
    }
  });

  let rx2 = rx.clone();
  tokio::spawn(async move {
    println!("rx2 started");
    loop {
      match rx2.recv().await {
        Ok(v2) => {
          println!("rx2 recv: {:?}", v2);
        }
        Err(e) => {
          println!("rx2 recv error: {:?}", e);
          break;
        }
      }
    }
  });

  println!("Rx count: {:?}", rx.receiver_count());
  tokio::time::sleep(Duration::from_secs(1)).await;
  for i in 0..6 {
    tx.send(i).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
  }
  tx.close().unwrap();
  tokio::time::sleep(Duration::from_millis(100)).await;
}

async fn demo_mea_mpsc() {
  let (tx, mut rx) = mpsc::unbounded();
  tx.send(1).unwrap();
  let v = rx.recv().await.unwrap();
  println!("v: {:?}", v);
}
