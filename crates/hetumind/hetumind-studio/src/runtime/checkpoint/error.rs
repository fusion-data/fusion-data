use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckpointError {
  #[error("检查点存储错误: {0}")]
  StorageError(#[from] Box<dyn std::error::Error + Send + Sync>),
}
