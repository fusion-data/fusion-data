use std::net::AddrParseError;

use config::ConfigError;
use fusion_common::ctx::CtxError;
use serde::Serialize;
use serde_json::json;

use crate::{configuration::ConfigureError, security::Error as SecurityError};

#[derive(Debug, Serialize)]
pub struct DataError {
  pub code: i32,
  pub msg: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<serde_json::Value>,
  #[serde(skip)]
  pub source: Option<Box<dyn core::error::Error + Send + Sync>>,
}

impl fusion_common::DataError for DataError {
  fn code(&self) -> i32 {
    self.code
  }

  fn msg(&self) -> &str {
    &self.msg
  }

  fn data(&self) -> Option<&serde_json::Value> {
    self.data.as_ref()
  }

  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    self.source.as_ref().map(|e| &**e as &(dyn core::error::Error + 'static))
  }
}

impl core::error::Error for DataError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    self.source.as_ref().map(|e| &**e as &(dyn core::error::Error + 'static))
  }
}

impl core::fmt::Display for DataError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.code, self.msg)
  }
}

impl DataError {
  pub fn bad_request(msg: impl Into<String>) -> Self {
    Self { code: 400, msg: msg.into(), data: None, source: None }
  }

  pub fn not_found(msg: impl Into<String>) -> Self {
    Self { code: 404, msg: msg.into(), data: None, source: None }
  }

  pub fn conflicted(msg: impl Into<String>) -> Self {
    Self { code: 409, msg: msg.into(), data: None, source: None }
  }

  pub fn unauthorized(msg: impl Into<String>) -> Self {
    Self { code: 401, msg: msg.into(), data: None, source: None }
  }

  pub fn forbidden(msg: impl Into<String>) -> Self {
    Self { code: 403, msg: msg.into(), data: None, source: None }
  }

  pub fn server_error(msg: impl Into<String>) -> Self {
    Self { code: 500, msg: msg.into(), data: None, source: None }
  }

  pub fn biz_error(code: i32, msg: impl Into<String>, data: Option<serde_json::Value>) -> Self {
    Self { code, msg: msg.into(), data, source: None }
  }

  pub fn internal(
    code: i32,
    msg: impl Into<String>,
    source: Option<Box<dyn core::error::Error + Send + Sync>>,
  ) -> Self {
    Self { code, msg: msg.into(), data: None, source }
  }

  pub fn retry_limit(msg: impl Into<String>, retry_limit: u32) -> Self {
    let detail = json!({ "retry_limit": retry_limit });
    Self { code: 1429, msg: msg.into(), data: Some(detail), source: None }
  }
}

impl From<fusion_common::Error> for DataError {
  fn from(value: fusion_common::Error) -> Self {
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

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for DataError
where
  T: Send + Sync + 'static,
{
  fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
    let compatible_error: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(e);
    DataError::internal(500, "channel send error", Some(compatible_error))
  }
}

impl From<tokio::sync::oneshot::error::RecvError> for DataError {
  fn from(e: tokio::sync::oneshot::error::RecvError) -> Self {
    // tokio::sync::oneshot::error::RecvError implements Send + Sync
    let compatible_error: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(e);
    DataError::internal(500, "channel recv error", Some(compatible_error))
  }
}

impl From<tokio::task::JoinError> for DataError {
  fn from(value: tokio::task::JoinError) -> Self {
    // tokio::task::JoinError implements Send + Sync
    let compatible_error: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(value);
    DataError::internal(500, "Join tokio task error", Some(compatible_error))
  }
}

impl From<ConfigError> for DataError {
  fn from(value: ConfigError) -> Self {
    DataError::server_error(format!("Config load error: {}", value))
  }
}

impl From<AddrParseError> for DataError {
  fn from(value: AddrParseError) -> Self {
    DataError::server_error(format!("Addr parse error: {}", value))
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

#[cfg(feature = "with-uuid")]
impl From<uuid::Error> for DataError {
  fn from(value: uuid::Error) -> Self {
    DataError::internal(500, value.to_string(), None)
  }
}

impl From<ConfigureError> for DataError {
  fn from(value: ConfigureError) -> Self {
    DataError::server_error(value.to_string())
  }
}

#[cfg(feature = "tonic")]
impl From<protobuf::ParseError> for DataError {
  fn from(value: protobuf::ParseError) -> Self {
    DataError::biz_error(400, format!("Protobuf parse error: {}", value), None)
  }
}

#[cfg(feature = "tonic")]
impl From<protobuf::SerializeError> for DataError {
  fn from(value: protobuf::SerializeError) -> Self {
    // protobuf::SerializeError might not implement Send + Sync, so we'll convert it to a string
    let source: Option<Box<dyn core::error::Error + Send + Sync>> = Some(Box::new(value));
    DataError::internal(500, "Protobuf serialize error", source)
  }
}

#[cfg(feature = "tonic")]
impl From<tonic::transport::Error> for DataError {
  fn from(value: tonic::transport::Error) -> Self {
    DataError::server_error(format!("Grpc transport error: {}", value))
  }
}

#[cfg(feature = "tonic")]
impl From<tonic::Status> for DataError {
  fn from(value: tonic::Status) -> Self {
    // TODO 更精细的 gRPC 状态转换
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
      // TODO 存在 Ok -> DataError::internal 的转换吗？
      tonic::Code::Ok => DataError::internal(0, "", None),
    }
  }
}

#[cfg(feature = "tonic")]
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
    let mut status = tonic::Status::new(code, value.msg);
    // if let Some(detail) = value.detail {
    //   status.set_details(detail);
    // }
    if let Some(e) = value.source {
      // Convert the boxed error to an Arc to satisfy the set_source method
      // We need to use Arc::from to properly handle the conversion from Box<T> to Arc<T>
      let arc_error = std::sync::Arc::from(e);
      status.set_source(arc_error);
    }
    status
  }
}

#[cfg(feature = "fusionsql")]
impl From<fusionsql::SqlError> for DataError {
  fn from(value: fusionsql::SqlError) -> Self {
    match value {
      fusionsql::SqlError::Unauthorized(e) => DataError::unauthorized(e),
      fusionsql::SqlError::InvalidArgument { message } => DataError::bad_request(format!("InvalidArgument, {message}")),
      fusionsql::SqlError::EntityNotFound { schema, entity, id } => {
        DataError::not_found(format!("EntityNotFound, {}:{}:{}", schema.unwrap_or_default(), entity, id))
      }
      fusionsql::SqlError::NotFound { schema, table, sql } => {
        log::debug!("NotFound, schema: {}, table: {}, sql: {}", schema.unwrap_or_default(), table, sql);
        DataError::not_found(format!("NotFound, {}:{}", schema.unwrap_or_default(), table))
      }
      fusionsql::SqlError::ListLimitOverMax { max, actual } => {
        DataError::bad_request(format!("ListLimitOverMax, max: {max}, actual: {actual}"))
      }
      fusionsql::SqlError::ListLimitUnderMin { min, actual } => {
        DataError::bad_request(format!("ListLimitUnderMin, min: {min}, actual: {actual}"))
      }
      fusionsql::SqlError::ListPageUnderMin { min, actual } => {
        DataError::bad_request(format!("ListPageUnderMin, min: {min}, actual: {actual}"))
      }
      fusionsql::SqlError::UserAlreadyExists { key, value } => {
        DataError::conflicted(format!("UserAlreadyExists, {key}:{value}"))
      }
      fusionsql::SqlError::UniqueViolation { table, constraint } => {
        DataError::conflicted(format!("UniqueViolation, {table}:{constraint}"))
      }
      fusionsql::SqlError::ExecuteError { table, message } => {
        DataError::server_error(format!("ExecuteError, {}:{}", table, message))
      }
      fusionsql::SqlError::ExecuteFail { schema, table } => {
        DataError::server_error(format!("ExecuteFail, {:?}:{}", schema, table))
      }
      fusionsql::SqlError::CountFail { schema, table } => {
        DataError::server_error(format!("CountFail, {:?}:{}", schema, table))
      }
      e @ fusionsql::SqlError::InvalidDatabase(_) => DataError::server_error(e.to_string()),
      e @ fusionsql::SqlError::CantCreateModelManagerProvider(_) => DataError::server_error(e.to_string()),
      e @ fusionsql::SqlError::IntoSeaError(_) => DataError::server_error(e.to_string()),
      e @ fusionsql::SqlError::SeaQueryError(_) => DataError::server_error(e.to_string()),
      e @ fusionsql::SqlError::JsonError(_) => DataError::server_error(e.to_string()),
      fusionsql::SqlError::DbxError(e) => {
        // Convert to a compatible error that implements Send + Sync
        let error_msg = e.to_string();
        let compatible_error: Box<dyn std::error::Error + Send + Sync + 'static> =
          Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg));
        DataError::internal(500, "Dbx Error", Some(compatible_error))
      }
      fusionsql::SqlError::Sqlx(e) => {
        // Convert to a compatible error that implements Send + Sync
        let error_msg = e.to_string();
        let compatible_error: Box<dyn std::error::Error + Send + Sync + 'static> =
          Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg));
        DataError::internal(500, "Sqlx Error", Some(compatible_error))
      }
    }
  }
}

#[cfg(feature = "fusionsql")]
impl From<fusionsql::store::DbxError> for DataError {
  fn from(value: fusionsql::store::DbxError) -> Self {
    DataError::server_error(value.to_string())
  }
}

impl From<SecurityError> for DataError {
  fn from(value: SecurityError) -> Self {
    match value {
      SecurityError::TokenExpired => DataError::unauthorized("Token expired"),
      SecurityError::SignatureNotMatching => DataError::unauthorized("Signature not matching"),
      SecurityError::InvalidPassword => DataError::unauthorized("Invalid password"),
      SecurityError::FailedToVerifyPassword => DataError::unauthorized("Failed to verify password"),
      // SecurityError::HmacFailNewFromSlice => todo!(),
      // SecurityError::InvalidFormat => todo!(),
      // SecurityError::CannotDecodeIdent => todo!(),
      // SecurityError::CannotDecodeExp => todo!(),
      // SecurityError::ExpNotIso => todo!(),
      // SecurityError::FailedToHashPassword => todo!(),
      _ => DataError::server_error(value.to_string()),
    }
  }
}

impl<T> From<mea::mpsc::SendError<T>> for DataError {
  fn from(value: mea::mpsc::SendError<T>) -> Self {
    DataError::server_error(format!("Send to mea::mpsc error, {}", value))
  }
}
