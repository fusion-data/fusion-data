use thiserror::Error;

/// 队列错误类型
#[derive(Debug, Error)]
pub enum QueueError {
  #[error("连接错误: {0}")]
  ConnectionError(String),

  #[error("序列化错误: {0}")]
  SerializationError(String),

  #[error("任务不存在: {0}")]
  TaskNotFound(String),

  #[error("队列已满")]
  QueueFull,

  #[error("内部错误: {0}")]
  InternalError(String),
}

/// Worker 错误类型
#[derive(Debug, Error)]
pub enum WorkerError {
  #[error("执行错误: {0}")]
  ExecutionError(String),

  #[error("超时")]
  Timeout,

  #[error("取消")]
  Cancelled,
}
