use fusion_core::DataError;
use hetumind_core::workflow::{NodeExecutionError, TriggerError, ValidationError, WorkflowExecutionError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GuixuError {
  #[error("工作流执行错误: {0}")]
  WorkflowExecution(#[from] WorkflowExecutionError),

  #[error("节点执行错误: {0}")]
  NodeExecution(#[from] NodeExecutionError),

  #[error("触发器错误: {0}")]
  Trigger(#[from] TriggerError),

  #[error("验证错误: {0}")]
  Validation(#[from] ValidationError),

  #[error("序列化错误: {0}")]
  Serialization(#[from] serde_json::Error),

  #[error("数据库错误: {0}")]
  Database(#[from] sqlx::Error),

  #[error("IO 错误: {0}")]
  Io(#[from] std::io::Error),
}

impl From<GuixuError> for DataError {
  fn from(value: GuixuError) -> Self {
    match value {
      GuixuError::WorkflowExecution(e) => DataError::internal(500, e.to_string(), Some(Box::new(e))),
      GuixuError::NodeExecution(e) => DataError::internal(500, e.to_string(), Some(Box::new(e))),
      GuixuError::Trigger(e) => DataError::internal(500, e.to_string(), Some(Box::new(e))),
      GuixuError::Validation(e) => DataError::internal(500, e.to_string(), Some(Box::new(e))),
      GuixuError::Serialization(e) => DataError::internal(500, e.to_string(), Some(Box::new(e))),
      GuixuError::Database(e) => DataError::internal(500, e.to_string(), Some(Box::new(e))),
      GuixuError::Io(e) => DataError::internal(500, e.to_string(), Some(Box::new(e))),
    }
  }
}
