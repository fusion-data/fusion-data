use serde::Serialize;
use thiserror::Error;

use crate::ctx::CtxError;

pub type Result<T> = core::result::Result<T, Error>;
pub type DataResult<T> = core::result::Result<T, DataError>;

#[derive(Debug, Error)]
pub enum Error {
  // -- Base64
  #[error("Decode base64 fail, string is {0}")]
  FailToB64uDecode(String),

  #[error("Parse date fail, data is {0}")]
  DateFailParse(String),

  #[error("Key fail.")]
  KeyFail,

  #[error("Password not match.")]
  PwdNotMatching,

  #[error("Missing env: {0}")]
  MissingEnv(String),

  #[error("Wrong format: {0}")]
  WrongFormat(String),

  #[error("Failed to set env: {0}, value: {1}, error: {2}")]
  FailedToSetEnv(String, String, String),

  #[error("Failed to remove env: {0}, error: {1}")]
  FailedToRemoveEnv(String, String),
}

impl From<chrono::ParseError> for Error {
  fn from(value: chrono::ParseError) -> Self {
    Error::DateFailParse(value.to_string())
  }
}

/// 数据（业务）错误，兼容 jsonrpc error
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataError {
  pub code: i32,
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub detail: Option<serde_json::Value>,
  #[serde(skip)]
  pub source: Option<Box<dyn core::error::Error + Send + Sync>>,
}

impl core::error::Error for DataError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    self.source.as_ref().map(|e| &**e as &(dyn core::error::Error + 'static))
  }
}

impl core::fmt::Display for DataError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.code, self.message)
  }
}

impl DataError {
  pub fn bad_request(msg: impl Into<String>) -> Self {
    Self { code: 400, message: msg.into(), detail: None, source: None }
  }

  pub fn not_found(msg: impl Into<String>) -> Self {
    Self { code: 404, message: msg.into(), detail: None, source: None }
  }

  pub fn conflicted(msg: impl Into<String>) -> Self {
    Self { code: 409, message: msg.into(), detail: None, source: None }
  }

  pub fn unauthorized(msg: impl Into<String>) -> Self {
    Self { code: 401, message: msg.into(), detail: None, source: None }
  }

  pub fn forbidden(msg: impl Into<String>) -> Self {
    Self { code: 403, message: msg.into(), detail: None, source: None }
  }

  pub fn server_error(msg: impl Into<String>) -> Self {
    Self { code: 500, message: msg.into(), detail: None, source: None }
  }

  pub fn biz_error(code: i32, msg: impl Into<String>, data: Option<serde_json::Value>) -> Self {
    Self { code, message: msg.into(), detail: data, source: None }
  }

  pub fn internal(
    code: i32,
    msg: impl Into<String>,
    source: Option<Box<dyn core::error::Error + Send + Sync>>,
  ) -> Self {
    Self { code, message: msg.into(), detail: None, source }
  }

  pub fn retry_limit(msg: impl Into<String>, retry_limit: u32) -> Self {
    let detail = serde_json::json!({ "retry_limit": retry_limit });
    Self { code: 1429, message: msg.into(), detail: Some(detail), source: None }
  }
}

// ==========================================
// 基础 From 实现
// ==========================================

impl From<Error> for DataError {
  fn from(value: Error) -> Self {
    DataError::server_error(value.to_string())
  }
}

impl From<std::time::SystemTimeError> for DataError {
  fn from(value: std::time::SystemTimeError) -> Self {
    Self::internal(500, "SystemTimeError", Some(Box::new(value)))
  }
}

impl From<std::io::Error> for DataError {
  fn from(value: std::io::Error) -> Self {
    let error_msg = value.to_string();
    DataError::internal(500, format!("IO error: {}", error_msg), Some(Box::new(value)))
  }
}

impl From<serde_json::Error> for DataError {
  fn from(value: serde_json::Error) -> Self {
    DataError::internal(500, "JSON error", Some(Box::new(value)))
  }
}

impl From<CtxError> for DataError {
  fn from(value: CtxError) -> Self {
    match value {
      CtxError::Unauthorized(msg) => DataError::unauthorized(msg),
      CtxError::InvalidPayload => DataError::unauthorized("Invalid ctx payload"),
    }
  }
}

impl From<std::net::AddrParseError> for DataError {
  fn from(value: std::net::AddrParseError) -> Self {
    DataError::server_error(format!("Addr parse error: {}", value))
  }
}

#[cfg(feature = "with-uuid")]
impl From<uuid::Error> for DataError {
  fn from(value: uuid::Error) -> Self {
    DataError::internal(500, value.to_string(), None)
  }
}

// ==========================================
// Tokio 相关 (feature gated)
// ==========================================

#[cfg(feature = "with-tokio")]
impl<T> From<tokio::sync::mpsc::error::SendError<T>> for DataError
where
  T: Send + Sync + 'static,
{
  fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
    let compatible_error: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(e);
    DataError::internal(500, "channel send error", Some(compatible_error))
  }
}

#[cfg(feature = "with-tokio")]
impl From<tokio::sync::oneshot::error::RecvError> for DataError {
  fn from(e: tokio::sync::oneshot::error::RecvError) -> Self {
    let compatible_error: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(e);
    DataError::internal(500, "channel recv error", Some(compatible_error))
  }
}

#[cfg(feature = "with-tokio")]
impl From<tokio::task::JoinError> for DataError {
  fn from(value: tokio::task::JoinError) -> Self {
    let compatible_error: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(value);
    DataError::internal(500, "Join tokio task error", Some(compatible_error))
  }
}

// ==========================================
// MEA 相关 (feature gated)
// ==========================================

#[cfg(feature = "with-mea")]
impl<T> From<mea::mpsc::SendError<T>> for DataError {
  fn from(value: mea::mpsc::SendError<T>) -> Self {
    DataError::server_error(format!("Send to mea::mpsc error, {}", value))
  }
}

// ==========================================
// Config 相关 (feature gated)
// ==========================================

#[cfg(feature = "with-config")]
impl From<config::ConfigError> for DataError {
  fn from(value: config::ConfigError) -> Self {
    DataError::server_error(format!("Config load error: {}", value))
  }
}

// ==========================================
// Tonic/gRPC 相关 (feature gated)
// ==========================================

#[cfg(feature = "with-tonic")]
impl From<protobuf::ParseError> for DataError {
  fn from(value: protobuf::ParseError) -> Self {
    DataError::biz_error(400, format!("Protobuf parse error: {}", value), None)
  }
}

#[cfg(feature = "with-tonic")]
impl From<protobuf::SerializeError> for DataError {
  fn from(value: protobuf::SerializeError) -> Self {
    let source: Option<Box<dyn core::error::Error + Send + Sync>> = Some(Box::new(value));
    DataError::internal(500, "Protobuf serialize error", source)
  }
}

#[cfg(feature = "with-tonic")]
impl From<tonic::transport::Error> for DataError {
  fn from(value: tonic::transport::Error) -> Self {
    DataError::server_error(format!("Grpc transport error: {}", value))
  }
}

#[cfg(feature = "with-tonic")]
impl From<tonic::Status> for DataError {
  fn from(value: tonic::Status) -> Self {
    let msg = value.message();
    match value.code() {
      tonic::Code::Cancelled => DataError::server_error(msg),
      tonic::Code::Unknown => DataError::server_error(msg),
      tonic::Code::InvalidArgument => DataError::bad_request(msg),
      tonic::Code::DeadlineExceeded => DataError::server_error(msg),
      tonic::Code::NotFound => DataError::not_found(msg),
      tonic::Code::AlreadyExists => DataError::conflicted(msg),
      tonic::Code::PermissionDenied => DataError::server_error(msg),
      tonic::Code::ResourceExhausted => DataError::server_error(msg),
      tonic::Code::FailedPrecondition => DataError::forbidden(msg),
      tonic::Code::Aborted => DataError::server_error(msg),
      tonic::Code::OutOfRange => DataError::bad_request(msg),
      tonic::Code::Unimplemented => DataError::server_error(msg),
      tonic::Code::Internal => DataError::server_error(msg),
      tonic::Code::Unavailable => DataError::server_error(msg),
      tonic::Code::DataLoss => DataError::server_error(msg),
      tonic::Code::Unauthenticated => DataError::unauthorized(msg),
      tonic::Code::Ok => DataError::internal(0, "", None),
    }
  }
}

#[cfg(feature = "with-tonic")]
impl From<DataError> for tonic::Status {
  fn from(value: DataError) -> Self {
    let code = match value.code {
      400 => tonic::Code::InvalidArgument,
      401 => tonic::Code::Unauthenticated,
      403 => tonic::Code::PermissionDenied,
      404 => tonic::Code::NotFound,
      409 => tonic::Code::Aborted,
      413 => tonic::Code::ResourceExhausted,
      429 => tonic::Code::Unavailable,
      500 => tonic::Code::Internal,
      501 => tonic::Code::Unimplemented,
      503 => tonic::Code::Unavailable,
      504 => tonic::Code::DeadlineExceeded,
      505 => tonic::Code::Unavailable,
      0 | (200..=299) => tonic::Code::Ok,
      _ => tonic::Code::Unknown,
    };
    let mut status = tonic::Status::new(code, value.message);
    if let Some(e) = value.source {
      let arc_error = std::sync::Arc::from(e);
      status.set_source(arc_error);
    }
    status
  }
}
