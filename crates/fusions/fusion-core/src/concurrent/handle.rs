use tokio::task::JoinHandle;

use crate::DataError;

pub struct ServiceHandle<T = ()> {
  name: String,
  handle: JoinHandle<T>,
}

impl<T> ServiceHandle<T> {
  pub fn new(name: impl Into<String>, handle: JoinHandle<T>) -> Self {
    Self { name: name.into(), handle }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  /// Wait for the service to complete and return the result.
  ///
  /// # Returns
  ///
  /// * `Ok((name, res))` - The service name and result.
  /// * `Err((name, e))` - The service panicked or was cancelled.
  pub async fn await_complete(self) -> Result<(String, T), (String, DataError)> {
    let name = self.name;
    match self.handle.await {
      Ok(r) => Ok((name, r)),
      Err(e) => Err((name, DataError::from(e))),
    }
  }
}
