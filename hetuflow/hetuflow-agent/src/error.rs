use fusion_core::DataError;
use thiserror::Error;

/// 任务执行错误类型
#[derive(Debug, Error)]
pub enum TaskExecutionError {
  #[error("Task cancelled")]
  Cancelled,
  #[error("Process start failed: {0}")]
  ProcessStartFailed(DataError),
  #[error("Process timeout")]
  ProcessTimeout,
  #[error("Process killed")]
  ProcessKilled,
  #[error("Resource exhausted")]
  ResourceExhausted,
  #[error("Dependency check failed")]
  DependencyCheckFailed,
  #[error("Configuration error")]
  ConfigurationError,
  #[error("Network error")]
  NetworkError,
  #[error("Failed")]
  Failed,
}
