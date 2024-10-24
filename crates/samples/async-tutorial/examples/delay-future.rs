use futures::future::poll_fn;
use std::{
  future::Future,
  pin::Pin,
  task::Poll,
  time::{Duration, Instant},
};

use async_tutorial::future::Delay;

#[tokio::main]
async fn main() {
  let when = Instant::now() + Duration::from_millis(10);
  // let future = Delay { when, poll_count: 0 };

  // let (out, poll_count) = future.await;
  // assert_eq!(out, "done");
  // println!("Poll count is {}", poll_count);
  let mut delay = Some(Delay { when, waker: None, poll_count: 0 });

  poll_fn(move |cx| {
    let mut delay = delay.take().unwrap();
    let res = Pin::new(&mut delay).poll(cx);
    assert!(res.is_pending());
    tokio::spawn(async move {
      delay.await;
    });

    Poll::Ready(())
  })
  .await;
}
