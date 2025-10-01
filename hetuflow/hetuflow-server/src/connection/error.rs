use fusion_core::DataError;
use fusion_web::WebError;
use log::{error, warn};
use fusionsql::SqlError;
use thiserror::Error;

/// Gateway 模块专用错误类型
#[derive(Debug, Error)]
pub enum GatewayError {
  #[error("Connection not found: {agent_id}")]
  ConnectionNotFound { agent_id: String },

  #[error("Async queue error: {0}")]
  AsyncQueueError(String),

  #[error("Authentication failed: {reason}")]
  AuthenticationFailed { reason: String },

  #[error("Message routing failed: {reason}")]
  MessageRoutingFailed { reason: String },

  #[error("Serialization error: {0}")]
  Serialization(#[from] serde_json::Error),

  #[error("Database error: {0}")]
  Database(#[from] SqlError),
}

impl GatewayError {
  pub fn async_queue_error(reason: impl Into<String>) -> Self {
    GatewayError::AsyncQueueError(reason.into())
  }

  /// 转换为 HTTP 状态码
  pub fn to_http_status(&self) -> u16 {
    match self {
      GatewayError::AuthenticationFailed { .. } => 401,
      GatewayError::ConnectionNotFound { .. } => 404,
      GatewayError::Database(_) => 500,
      GatewayError::Serialization(_) => 400,
      GatewayError::MessageRoutingFailed { .. } => 500,
      GatewayError::AsyncQueueError(_) => 503,
    }
  }

  /// 记录错误日志
  pub fn log_error(&self) {
    match self {
      GatewayError::Database(data_error) => {
        error!("Database error in Gateway: {:?}", data_error);
      }
      GatewayError::AuthenticationFailed { reason } => {
        warn!("Authentication failed: {}", reason);
      }
      _ => {
        error!("Gateway error: {:?}", self);
      }
    }
  }

  /// 创建连接未找到错误
  pub fn connection_not_found(agent_id: String) -> Self {
    GatewayError::ConnectionNotFound { agent_id }
  }

  /// 创建认证失败错误
  pub fn authentication_failed(reason: impl Into<String>) -> Self {
    GatewayError::AuthenticationFailed { reason: reason.into() }
  }

  /// 创建消息路由失败错误
  pub fn message_routing_failed(reason: impl Into<String>) -> Self {
    GatewayError::MessageRoutingFailed { reason: reason.into() }
  }

  pub fn internal(reason: impl Into<String>) -> Self {
    GatewayError::MessageRoutingFailed { reason: reason.into() }
  }
}

/// 与 fusion_core::DataError 的兼容性转换
impl From<GatewayError> for DataError {
  fn from(err: GatewayError) -> Self {
    match err {
      GatewayError::ConnectionNotFound { agent_id } => {
        DataError::not_found(format!("Agent connection not found: {}", agent_id))
      }
      GatewayError::AuthenticationFailed { reason } => DataError::unauthorized(reason),
      GatewayError::Serialization(json_error) => DataError::bad_request(format!("Serialization error: {}", json_error)),
      e => DataError::server_error(e.to_string()),
    }
  }
}

/// 与 fusion_web::WebError 的兼容性置换
impl From<GatewayError> for WebError {
  fn from(err: GatewayError) -> Self {
    match err {
      GatewayError::ConnectionNotFound { agent_id } => {
        WebError::new_with_code(404, format!("Agent connection not found: {}", agent_id))
      }
      GatewayError::AuthenticationFailed { reason } => WebError::new_with_code(401, reason),
      GatewayError::Serialization(json_error) => {
        WebError::new_with_code(400, format!("Serialization error: {}", json_error))
      }
      GatewayError::MessageRoutingFailed { reason } => {
        WebError::new_with_code(400, format!("Message routing failed: {}", reason))
      }
      GatewayError::AsyncQueueError(reason) => {
        error!("[Gateway] Async queue error: {}", reason);
        WebError::new_with_code(503, reason)
      }
      GatewayError::Database(sql_error) => {
        error!("[Gateway] Database error: {:?}", sql_error);
        WebError::new_with_code(500, format!("Database error: {}", sql_error))
      }
    }
  }
}
