use serde::Deserialize;

mod postgres_queue;
mod queue_plugin;
mod queue_provider;
mod task_queue_entity;
mod utils;

pub use postgres_queue::{PostgresQueue, PostgresQueueConfig};
pub use queue_plugin::QueueProviderPlugin;
pub use queue_provider::QueueProvider;
pub use task_queue_entity::TaskQueueEntity;
pub use utils::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum QueueConfig {
  Postgres {
    #[serde(default)]
    postgres: PostgresQueueConfig,
  },
}

impl Default for QueueConfig {
  fn default() -> Self {
    QueueConfig::Postgres { postgres: PostgresQueueConfig::default() }
  }
}
