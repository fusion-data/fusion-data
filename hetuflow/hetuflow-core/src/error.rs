use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::protocol::TaskExecutionError;

/// WebSocket 错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum WebSocketError {
  #[error("连接错误: {0}")]
  ConnectionError(String),
  #[error("消息错误: {0}")]
  MessageError(String),
  #[error("序列化错误: {0}")]
  SerializationError(String),
  #[error("认证错误: {0}")]
  AuthenticationError(String),
  #[error("超时错误")]
  TimeoutError,
  #[error("未知错误: {0}")]
  UnknownError(String),
}
