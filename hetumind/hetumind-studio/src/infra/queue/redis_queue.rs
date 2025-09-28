use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct RedisQueueConfig {
  pub url: String,
  pub max_pool_size: u32,
  pub visibility_timeout: Duration,
  pub enable_listen_notify: bool,
}

impl Default for RedisQueueConfig {
  fn default() -> Self {
    Self {
      url: "redis://localhost:6379".to_string(),
      max_pool_size: 10,
      visibility_timeout: Duration::from_secs(30),
      enable_listen_notify: false,
    }
  }
}

pub struct RedisQueue {
  pub config: RedisQueueConfig,
}
