use std::{
  future::Future,
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll, Waker},
  thread,
  time::{Duration, Instant},
};

use futures::Stream;
use tokio::sync::Notify;

// Implementing the delay function using Notify in tokio
pub async fn delay(dur: Duration) {
  let when = Instant::now() + dur;
  let notify = Arc::new(Notify::new());
  let notify_clone = notify.clone();

  thread::spawn(move || {
    let now = Instant::now();
    if now < when {
      thread::sleep(when - now);
    }
    notify_clone.notify_one();
  });

  notify.notified().await;
}

pub struct Delay {
  pub when: Instant,
  // This is Some when we have spawned a thread, and None otherwise.
  pub waker: Option<Arc<Mutex<Waker>>>,
  pub poll_count: usize,
}

impl Future for Delay {
  type Output = (&'static str, usize);

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if Instant::now() >= self.when {
      println!("Hello world");
      return Poll::Ready(("done", self.poll_count));
    }

    println!("Not yet, waiting...");
    self.poll_count += 1;

    // The duration has not elapsed. If this is the first time the future is called,
    // spawn the timer thread. If the timer thread is already running, ensure the
    // stored `Waker` matches the current task's waker.
    if let Some(waker) = &self.waker {
      let mut waker = waker.lock().unwrap();

      // Check if the stored waker matches the current task's waker.
      // This is necessary as the `Delay` future instance may move to
      // a different task between calls to `poll`. If this happens, the
      // waker contained by the given `Context` will differ and we must
      // update our stored waker to reflect this change.
      if !waker.will_wake(cx.waker()) {
        *waker = cx.waker().clone();
      }
    } else {
      // Get a handle to the waker for the current task
      let when = self.when;
      let waker = Arc::new(Mutex::new(cx.waker().clone()));
      self.waker = Some(waker.clone());

      // This is the first time `poll` is called, spawn the timer thread.
      thread::spawn(move || {
        let now = Instant::now();

        if now < when {
          thread::sleep(when - now);
        }

        // The duration has elapsed. Notify the caller by invoking the waker.
        let waker = waker.lock().unwrap();
        waker.wake_by_ref();
      });
    }

    // By now, the waker is stored and the timer thread is started. The duration
    // has elapsed (recall that we checked for this first thing), ergo the future
    // has not completed so we must return `Poll::Pending`.
    //
    // The `Future` trait contract requires that when `Pending` is returned, the
    // future ensures that the given waker is signalled once the future should be
    // polled again. In our case, by returning `Pending` here, we are promising
    // that we will invoke the given waker included in the `Context` argument once
    // the requested duration has elapsed. We ensure this by spawning the timer
    // thread above.
    //
    // If we forget to invoke the waker, the task will hang indefinitely.
    Poll::Pending
  }
}

pub enum MainFuture {
  // Initialized, never polled
  State0,
  // Waiting on `Delay`, i.e. the `future.await` line.
  State1(Delay),
  // The future has completed.
  Terminated,
}

impl Future for MainFuture {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    use MainFuture::*;

    loop {
      match *self {
        State0 => {
          let when = Instant::now() + Duration::from_millis(10);
          let future = Delay { when, waker: Some(Arc::new(Mutex::new(cx.waker().clone()))), poll_count: 0 };
          *self = State1(future);
        }
        State1(ref mut my_future) => match Pin::new(my_future).poll(cx) {
          Poll::Ready((out, _)) => {
            assert_eq!(out, "done");
            *self = Terminated;
            return Poll::Ready(());
          }
          Poll::Pending => return Poll::Pending,
        },
        Terminated => panic!("future polled after completion"),
      }
    }
  }
}

// ---- Interval ----
pub struct Interval {
  rem: usize,
  delay: Delay,
}

impl Default for Interval {
  fn default() -> Self {
    Self { rem: 3, delay: Delay { when: Instant::now(), waker: None, poll_count: 0 } }
  }
}

impl Stream for Interval {
  type Item = ();

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    if self.rem == 0 {
      return Poll::Ready(None);
    }

    match Pin::new(&mut self.delay).poll(cx) {
      Poll::Ready(_) => {
        let when = self.delay.when + Duration::from_millis(10);
        self.delay =
          Delay { when, waker: Some(Arc::new(Mutex::new(cx.waker().clone()))), poll_count: self.delay.poll_count };
        self.rem -= 1;
        Poll::Ready(Some(()))
      }
      Poll::Pending => Poll::Pending,
    }
  }
}
