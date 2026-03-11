mod handle;
mod task;

pub use handle::ServiceHandle;
pub use task::{RetryStrategy, ServiceTask};

use crate::DataError;

pub struct TaskResult<T = ()> {
  pub result: T,
  pub retry_count: u32,
}

impl<T> TaskResult<T> {
  pub fn new(result: T, retry_count: u32) -> Self {
    Self { result, retry_count }
  }
}

impl TaskResult {
  pub fn empty() -> Self {
    Self::new((), 0)
  }
}

pub type TaskServiceHandle = ServiceHandle<Result<TaskResult, DataError>>;
