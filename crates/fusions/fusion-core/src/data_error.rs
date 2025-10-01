use std::{net::AddrParseError, num::ParseIntError};

use config::ConfigError;
use fusion_corelib::ctx::CtxError;
use serde::{Serialize, ser::SerializeMap};
use serde_json::json;
use thiserror::Error;

use crate::{configuration::ConfigureError, security::Error as SecurityError};

#[derive(Error, Debug)]
pub enum DataError {
  #[error("Biz error. code: {code}, msg: {msg}")]
  BizError { code: i32, msg: String, detail: Option<Box<serde_json::Value>> },

  #[error("Internal error: {code} {msg}")]
  InternalError { code: i32, msg: String, cause: Option<Box<dyn std::error::Error + Send + Sync>> },

  #[error(transparent)]
  SystemTimeError(#[from] std::time::SystemTimeError),

  #[error(transparent)]
  ParseIntError(#[from] ParseIntError),

  #[error(transparent)]
  IoError(#[from] std::io::Error),

  #[error(transparent)]
  JsonError(#[from] serde_json::Error),
}

impl DataError {
  pub fn bad_request(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 400, msg: msg.into(), detail: None }
  }

  pub fn not_found(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 404, msg: msg.into(), detail: None }
  }

  pub fn conflicted(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 409, msg: msg.into(), detail: None }
  }

  pub fn unauthorized(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 401, msg: msg.into(), detail: None }
  }

  pub fn forbidden(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 403, msg: msg.into(), detail: None }
  }

  pub fn server_error(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 500, msg: msg.into(), detail: None }
  }

  pub fn internal(code: i32, msg: impl Into<String>, cause: Option<Box<dyn std::error::Error + Send + Sync>>) -> Self {
    DataError::InternalError { code, msg: msg.into(), cause }
  }

  pub fn retry_limit(msg: impl Into<String>, retry_limit: u32) -> Self {
    let detail = json!({ "retry_limit": retry_limit });
    DataError::BizError { code: 1429, msg: msg.into(), detail: Some(Box::new(detail)) }
  }
}

impl From<fusion_common::Error> for DataError {
  fn from(value: fusion_common::Error) -> Self {
    DataError::server_error(value.to_string())
  }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for DataError
where
  T: Send + Sync + 'static,
{
  fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
    DataError::InternalError { code: 500, msg: "channel send error".into(), cause: Some(Box::new(e)) }
  }
}

impl From<tokio::sync::oneshot::error::RecvError> for DataError {
  fn from(e: tokio::sync::oneshot::error::RecvError) -> Self {
    DataError::InternalError { code: 500, msg: "channel recv error".into(), cause: Some(Box::new(e)) }
  }
}

impl From<tokio::task::JoinError> for DataError {
  fn from(value: tokio::task::JoinError) -> Self {
    DataError::InternalError { code: 500, msg: "Join tokio task error".into(), cause: Some(Box::new(value)) }
  }
}

impl From<ConfigError> for DataError {
  fn from(value: ConfigError) -> Self {
    DataError::server_error(format!("Config load error: {:?}", value.to_string()))
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
    DataError::InternalError { code: 500, msg: value.to_string(), cause: None }
  }
}

impl Serialize for DataError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut map = serializer.serialize_map(Some(2))?;
    // TODO
    map.serialize_entry("aa", "error")?;
    map.end()
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
    DataError::BizError { code: 400, msg: format!("Protobuf parse error: {}", value), detail: None }
  }
}

#[cfg(feature = "tonic")]
impl From<protobuf::SerializeError> for DataError {
  fn from(value: protobuf::SerializeError) -> Self {
    DataError::InternalError {
      code: 500,
      msg: format!("Protobuf serialize error: {}", value),
      cause: Some(Box::new(value)),
    }
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
    match value {
      DataError::BizError { code, msg, .. } => make_tonic_status(code, msg),
      DataError::InternalError { code, msg, .. } => make_tonic_status(code, msg),
      DataError::SystemTimeError(ex) => tonic::Status::from_error(ex.into()),
      DataError::ParseIntError(ex) => tonic::Status::from_error(ex.into()),
      DataError::IoError(e) => tonic::Status::internal(e.to_string()),
      DataError::JsonError(ex) => tonic::Status::from_error(ex.into()),
    }
  }
}

#[cfg(feature = "tonic")]
fn make_tonic_status(code: i32, msg: String) -> tonic::Status {
  if code == 0 || (200..300).contains(&code) {
    return tonic::Status::ok(msg);
  }

  if code == 400 {
    return tonic::Status::invalid_argument(msg);
  }

  if code == 401 {
    return tonic::Status::unauthenticated(msg);
  }

  if code == 403 {
    return tonic::Status::permission_denied(msg);
  }

  if code == 404 {
    return tonic::Status::not_found(msg);
  }

  if code == 409 {
    return tonic::Status::already_exists(msg);
  }

  if code == 501 {
    return tonic::Status::unimplemented(msg);
  }

  tonic::Status::internal(msg)
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
        DataError::InternalError { code: 500, msg: "Dbx Error".to_string(), cause: Some(Box::new(e)) }
      }
      fusionsql::SqlError::Sqlx(e) => {
        DataError::InternalError { code: 500, msg: "Sqlx Error".to_string(), cause: Some(Box::new(e)) }
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
