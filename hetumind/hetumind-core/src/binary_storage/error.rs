//! 二进制数据存储错误类型

use thiserror::Error;

/// 二进制数据存储错误类型
#[derive(Debug, Error)]
pub enum BinaryStorageError {
  #[error("存储配置错误: {0}")]
  ConfigError(String),

  #[error("存储操作错误: {0}")]
  OperationError(String),

  #[error("文件不存在: {0}")]
  FileNotFound(String),

  #[error("权限不足: {0}")]
  PermissionDenied(String),

  #[error("IO错误: {0}")]
  IoError(#[from] std::io::Error),

  #[error("序列化错误: {0}")]
  SerializationError(#[from] serde_json::Error),

  #[error("生命周期管理错误: {0}")]
  LifecycleError(String),

  #[error("指标收集错误: {0}")]
  MetricsError(String),
}

impl BinaryStorageError {
  /// 创建配置错误
  pub fn config<T: Into<String>>(msg: T) -> Self {
    Self::ConfigError(msg.into())
  }

  /// 创建操作错误
  pub fn operation<T: Into<String>>(msg: T) -> Self {
    Self::OperationError(msg.into())
  }

  /// 创建文件不存在错误
  pub fn file_not_found<T: Into<String>>(path: T) -> Self {
    Self::FileNotFound(path.into())
  }

  /// 创建权限不足错误
  pub fn permission_denied<T: Into<String>>(msg: T) -> Self {
    Self::PermissionDenied(msg.into())
  }

  /// 创建生命周期管理错误
  pub fn lifecycle<T: Into<String>>(msg: T) -> Self {
    Self::LifecycleError(msg.into())
  }

  /// 创建指标收集错误
  pub fn metrics<T: Into<String>>(msg: T) -> Self {
    Self::MetricsError(msg.into())
  }
}
