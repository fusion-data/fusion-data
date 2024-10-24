use std::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
};

use tokio::sync::{mpsc, oneshot};

struct MySelect {
  rx1: oneshot::Receiver<&'static str>,
  rx2: oneshot::Receiver<&'static str>,
}

impl Future for MySelect {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
    if let Poll::Ready(val) = Pin::new(&mut self.rx1).poll(cx) {
      println!("rx1 completed first with {:?}", val);
      return Poll::Ready(());
    }

    if let Poll::Ready(val) = Pin::new(&mut self.rx2).poll(cx) {
      println!("rx2 completed first with {:?}", val);
      return Poll::Ready(());
    }

    Poll::Pending
  }
}

#[tokio::main]
async fn main() {
  let (tx1, mut rx1) = mpsc::channel::<Option<&str>>(128);
  let (tx2, mut rx2) = mpsc::channel::<Option<&str>>(128);

  // use tx1 and tx2
  tokio::spawn(async move {
    // tx2.send(Some("two")).await.unwrap();
    // tx2.closed().await;
    drop(tx2);
  });
  tokio::spawn(async move {
    // tx1.send(Some("one")).await.unwrap();
    // tx1.closed().await;
    drop(tx1);
  });

  // MySelect { rx1, rx2 }.await;

  tokio::select! {
      Some(v) = rx1.recv() => {
          println!("Got {:?} from rx1", v);
      }
      Some(v) = rx2.recv() => {
          println!("Got {:?} from rx2", v);
      }
      else => {
          println!("Both channels closed");
      }
  }
}

/// select! 语法模式有3部分: <pattern> = <async expression> => <handler>
/// handler 部分将串行执行
async fn sync_handler() {
  let (tx1, rx1) = oneshot::channel();
  let (tx2, rx2) = oneshot::channel();

  let mut out = String::new();

  tokio::spawn(async move {
    tx1.send("one").unwrap();
  });
  tokio::spawn(async move {
    tx2.send("two").unwrap();
  });

  tokio::select! {
      _ = rx1 => {
          out.push_str("rx1 completed");
      }
      _ = rx2 => {
          out.push_str("rx2 completed");
      }
  }

  println!("{}", out);
}
