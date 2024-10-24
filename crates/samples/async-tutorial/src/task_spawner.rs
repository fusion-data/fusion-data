use tokio::runtime::Builder;
use tokio::sync::mpsc;

pub struct Task {
  name: String,
  // info that describes the task
}

async fn handle_task(task: Task) {
  println!("Got task {}", task.name);
}

#[derive(Clone)]
pub struct TaskSpawner {
  spawn: mpsc::Sender<Task>,
}

impl TaskSpawner {
  pub fn new() -> TaskSpawner {
    // Set up a channel for communicating.
    let (send, mut recv) = mpsc::channel(16);

    // Build the runtime for the new thread.
    //
    // The runtime is created before spawning the thread
    // to more cleanly forward errors if the `unwrap()`
    // panics.
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    std::thread::spawn(move || {
      rt.block_on(async move {
        while let Some(task) = recv.recv().await {
          tokio::spawn(handle_task(task));
        }

        // Once all senders have gone out of scope,
        // the `.recv()` call returns None and it will
        // exit from the while loop and shut down the
        // thread.
      });
    });

    TaskSpawner { spawn: send }
  }

  pub fn spawn_task(&self, task: Task) {
    match self.spawn.blocking_send(task) {
      Ok(()) => {}
      Err(_) => panic!("The shared runtime has shut down."),
    }
  }
}

impl Default for TaskSpawner {
  fn default() -> Self {
    Self::new()
  }
}
