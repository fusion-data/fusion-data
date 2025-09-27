use std::{future::Future, time::Duration};

use crate::DataError;

use super::{ServiceHandle, TaskResult};

#[derive(Debug)]
pub struct RetryStrategy {
  enable: bool,
  retry_limit: u32,
  interval: Duration,
  increase_rate: Option<f64>,
}

impl Default for RetryStrategy {
  fn default() -> Self {
    Self { enable: false, retry_limit: 5, interval: Duration::from_secs(30), increase_rate: None }
  }
}

impl RetryStrategy {
  pub fn new_disable() -> Self {
    Self::default().with_disable()
  }

  pub fn new_enable() -> Self {
    Self::default().with_enable()
  }

  pub fn enable(&self) -> bool {
    self.enable
  }

  pub fn retry_limit(&self) -> u32 {
    self.retry_limit
  }

  pub fn interval(&self) -> Duration {
    self.interval
  }

  pub fn increase_rate(&self) -> Option<f64> {
    self.increase_rate
  }

  pub fn with_enable(mut self) -> Self {
    self.enable = true;
    self
  }

  pub fn with_disable(mut self) -> Self {
    self.enable = false;
    self
  }

  pub fn with_retry_limit(mut self, retry_limit: u32) -> Self {
    self.retry_limit = retry_limit;
    self
  }

  pub fn with_interval(mut self, interval: Duration) -> Self {
    self.interval = interval;
    self
  }

  pub fn with_increase_rate(mut self, increase_rate: f64) -> Self {
    self.increase_rate = Some(increase_rate);
    self
  }
}

pub trait ServiceTask<T>
where
  T: Send + 'static,
{
  fn name(&self) -> &str {
    std::any::type_name_of_val(self)
  }

  fn retry_strategy(&self) -> RetryStrategy {
    RetryStrategy::default()
  }

  fn start(mut self) -> ServiceHandle<Result<TaskResult<T>, DataError>>
  where
    Self: Send + Sized + 'static,
  {
    let name = self.name().to_string();
    let name2 = name.clone();
    let retry_strategy = self.retry_strategy();
    let handle = tokio::spawn(async move {
      let mut retry_count = 0;
      let mut duration = retry_strategy.interval();
      let retry_limit = retry_strategy.retry_limit();
      loop {
        match self.run_loop().await {
          Ok(result) => {
            log::info!("The ServiceTask: [{}] has been executed successfully after {} retries", &name, retry_count);
            return Ok(TaskResult { result, retry_count });
          }
          Err(err) => {
            retry_count += 1;
            if retry_count > retry_limit {
              log::error!(
                "The ServiceTask: [{}] stop after {} retries has reached the retry limit: {}",
                &name,
                retry_count,
                retry_limit
              );
              return Err(err);
            }
            if let Some(increase_rate) = retry_strategy.increase_rate() {
              duration = Duration::from_secs_f64(duration.as_secs_f64() * increase_rate);
            }
            tokio::time::sleep(duration).await;
          }
        }
      }
    });
    ServiceHandle::new(name2, handle)
  }

  fn run_loop(&mut self) -> impl Future<Output = Result<T, DataError>> + Send;
}
