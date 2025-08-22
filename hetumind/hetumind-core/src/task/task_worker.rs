use async_trait::async_trait;
use uuid::Uuid;

use super::{QueueTask, TaskResult, WorkerError};

#[async_trait]
pub trait TaskWorker: Send + Sync {
  /// 处理任务
  async fn process_task(&self, task: &QueueTask) -> Result<TaskResult, WorkerError>;

  /// 获取 Worker ID
  fn worker_id(&self) -> &Uuid;

  /// 获取批处理大小
  fn batch_size(&self) -> usize {
    5
  }

  /// 是否应该停止
  fn should_stop(&self) -> bool;
}
