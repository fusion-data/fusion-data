use std::time::Duration;

use async_trait::async_trait;
use uuid::Uuid;

use super::{QueueError, QueueStats, QueueTask, TaskResult, TaskStatus};

/// 任务队列抽象接口
#[async_trait]
pub trait TaskQueue: Send + Sync {
  /// 初始化队列
  async fn initialize(&self) -> Result<(), QueueError>;

  /// 入队任务
  async fn enqueue(&self, task: QueueTask) -> Result<Uuid, QueueError>;

  /// 批量入队
  async fn enqueue_batch(&self, tasks: Vec<QueueTask>) -> Result<Vec<Uuid>, QueueError>;

  /// 出队任务（Worker 使用）
  async fn dequeue(&self, worker_id: &Uuid, batch_size: usize) -> Result<Vec<(Uuid, QueueTask)>, QueueError>;

  /// 确认任务完成
  async fn ack(&self, task_id: &Uuid, result: TaskResult) -> Result<(), QueueError>;

  /// 标记任务失败
  async fn nack(&self, task_id: &Uuid, error: &str, retry: bool) -> Result<(), QueueError>;

  /// 延迟任务
  async fn delay(&self, task_id: &Uuid, delay: Duration) -> Result<(), QueueError>;

  /// 取消任务
  async fn cancel(&self, task_id: &Uuid) -> Result<(), QueueError>;

  /// 获取任务状态
  async fn get_task_status(&self, task_id: &Uuid) -> Result<Option<TaskStatus>, QueueError>;

  /// 获取队列统计
  async fn get_stats(&self) -> Result<QueueStats, QueueError>;

  /// 清理过期数据
  async fn cleanup(&self, retention: Duration) -> Result<u64, QueueError>;
}
