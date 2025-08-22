use log::error;
use tokio::task::JoinError;

#[inline]
pub fn handle_join_error<T, E>(ret: Result<Result<T, E>, JoinError>, task_name: &str)
where
  E: core::fmt::Display,
{
  match ret {
    Ok(ret) => {
      if let Err(err) = ret {
        error!("Asynchronous task '{}' error: {}", task_name, err);
      }
    }
    Err(err) => error!("Asynchronous task '{}' Join error: {}", task_name, err),
  }
}
