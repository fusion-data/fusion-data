use std::{ops::Deref, sync::Arc};

use hetumind_core::task::{QueueError, TaskQueue};
use modelsql::ModelManager;

#[cfg(feature = "with-redis")]
use super::RedisQueue;
use super::{PostgresQueue, QueueConfig};

#[derive(Clone)]
pub struct QueueProvider(Arc<dyn TaskQueue>);

impl Deref for QueueProvider {
  type Target = Arc<dyn TaskQueue>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl QueueProvider {
  /// 根据配置创建队列实例
  pub async fn create(config: QueueConfig, mm: ModelManager) -> Result<QueueProvider, QueueError> {
    match config {
      QueueConfig::Postgres { postgres } => {
        let queue = PostgresQueue::new(mm, postgres);
        queue.initialize().await?;
        Ok(QueueProvider(Arc::new(queue)))
      }
      #[cfg(feature = "with-redis")]
      QueueConfig::Redis { url, redis } => {
        let queue = RedisQueue::new(url, redis)?;
        queue.initialize().await?;
        Ok(QueueProvider(Arc::new(queue)))
      }
    }
  }

  pub fn inner(&self) -> Arc<dyn TaskQueue> {
    self.0.clone()
  }
}
