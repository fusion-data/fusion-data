use std::{
  future::Future,
  pin::Pin,
  sync::{mpsc, Arc, Mutex},
  task::{Context, Poll},
};

use futures::task::ArcWake;

pub struct Task {
  // The `Mutex` is to make `Task`implement `Sync`. Only one thread accesses `task_future`
  // at any given time. The `Mutex` is not required for correctness. Real Tokio does not use
  // a mutex here, but real Tokio has more lines of code than can fit in a single tuturial page
  task_future: Mutex<TaskFuture>,
  executor: mpsc::Sender<Arc<Task>>,
}

impl Task {
  pub fn schedule(self: &Arc<Self>) {
    let _ = self.executor.send(self.clone());
  }

  pub fn poll(self: Arc<Self>) {
    // Create a waker from the `Task` instance. This uses the `ArcWake` impl from above.
    let waker = futures::task::waker(self.clone());
    let mut cx = Context::from_waker(&waker);

    // No other thread ever tries to lock the task_future
    let mut task_future = self.task_future.try_lock().unwrap();

    // Poll the inner future
    task_future.poll(&mut cx);
  }

  // Spawns a new task with the given future.
  //
  // Initializes a new Task harness containing the given future and pushed it
  // onto the `sender`. The receiver half of the channel will get the task and
  // execute it.
  pub fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
  where
    F: Future<Output = ()> + Send + 'static,
  {
    let task = Arc::new(Task { task_future: Mutex::new(TaskFuture::new(future)), executor: sender.clone() });

    let _ = sender.send(task);
  }
}

impl ArcWake for Task {
  fn wake_by_ref(arc_self: &Arc<Self>) {
    arc_self.schedule()
  }
}

pub struct TaskFuture {
  future: Pin<Box<dyn Future<Output = ()> + Send>>,
  poll: Poll<()>,
}

impl TaskFuture {
  pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
    Self { future: Box::pin(future), poll: Poll::Pending }
  }

  pub fn poll(&mut self, cx: &mut Context<'_>) {
    // Spurious wake-ups are allowed, even after a future has returned `Ready`.
    // However, polling a future which has already returned `Ready` is *not* allowed.
    // For this reason we need to check that the future is still pending before we
    // call it. Failure to do so can lead to a panic.
    if self.poll.is_pending() {
      self.poll = self.future.as_mut().poll(cx);
    }
  }
}

pub struct MiniTokio {
  // tasks: VecDeque<Task>,
  scheduled: mpsc::Receiver<Arc<Task>>,
  sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {
  pub fn new() -> Self {
    let (sender, scheduled) = mpsc::channel();
    Self { sender, scheduled }
  }

  /// Spawn a future onto the mini-tokio instance.
  ///
  /// The given future is wrapped with the `Task` harness and pushed into the
  /// `scheduled` queue. The future will be executed when `run()` is called.
  pub fn spawn<F>(&mut self, future: F)
  where
    F: Future<Output = ()> + Send + 'static,
  {
    Task::spawn(future, &self.sender);
  }

  pub fn run(&mut self) {
    while let Ok(task) = self.scheduled.recv() {
      task.poll();
    }
  }
}
