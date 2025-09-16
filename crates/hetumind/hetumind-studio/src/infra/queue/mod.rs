use serde::Deserialize;

mod postgres_queue;
mod queue_plugin;
mod queue_provider;
#[cfg(feature = "with-redis")]
mod redis_queue;
mod task_queue_entity;
mod utils;

pub use postgres_queue::{PostgresQueue, PostgresQueueConfig};
pub use queue_plugin::QueueProviderPlugin;
pub use queue_provider::QueueProvider;
#[cfg(feature = "with-redis")]
pub use redis_queue::{RedisQueue, RedisQueueConfig};
pub use task_queue_entity::TaskQueueEntity;
pub use utils::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum QueueConfig {
  Postgres {
    #[serde(default)]
    postgres: PostgresQueueConfig,
  },
  #[cfg(feature = "with-redis")]
  Redis {
    url: String,
    #[serde(default)]
    redis: RedisQueueConfig,
  },
}

impl Default for QueueConfig {
  fn default() -> Self {
    QueueConfig::Postgres { postgres: PostgresQueueConfig::default() }
  }
}
