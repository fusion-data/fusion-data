use std::{net::AddrParseError, num::ParseIntError};

use config::ConfigError;
use serde::{Serialize, ser::SerializeMap};
use thiserror::Error;
use tracing::{error, subscriber::SetGlobalDefaultError};

use crate::{configuration::ConfigureError, security::Error as SecurityError};

#[derive(Error, Debug)]
pub enum DataError {
  #[error("Biz error. code: {code}, msg: {msg}")]
  BizError { code: i32, msg: String },

  #[error("Internal error: {code} {msg}")]
  InternalError { code: i32, msg: String, cause: Option<Box<dyn std::error::Error + Send + Sync>> },

  #[error(transparent)]
  SecurityError(#[from] SecurityError),

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
    DataError::BizError { code: 400, msg: msg.into() }
  }

  pub fn not_found(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 404, msg: msg.into() }
  }

  pub fn conflicted(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 409, msg: msg.into() }
  }

  pub fn server_error(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 500, msg: msg.into() }
  }

  pub fn unauthorized(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 401, msg: msg.into() }
  }

  pub fn forbidden(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 403, msg: msg.into() }
  }

  pub fn ok(msg: impl Into<String>) -> Self {
    DataError::BizError { code: 0, msg: msg.into() }
  }
}

impl From<SetGlobalDefaultError> for DataError {
  fn from(value: SetGlobalDefaultError) -> Self {
    DataError::InternalError { code: 500, msg: value.to_string(), cause: None }
  }
}

impl From<ultimate_common::Error> for DataError {
  fn from(value: ultimate_common::Error) -> Self {
    DataError::server_error(value.to_string())
  }
}

#[cfg(feature = "ultimate-api")]
impl From<ultimate_api::Error> for DataError {
  fn from(value: ultimate_api::Error) -> Self {
    match value {
      ultimate_api::Error::BizError { code, msg } => DataError::BizError { code, msg },
    }
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
impl From<prost::UnknownEnumValue> for DataError {
  fn from(value: prost::UnknownEnumValue) -> Self {
    DataError::BizError { code: 400, msg: format!("Unknown enum value: {}", value) }
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
      tonic::Code::Ok => DataError::ok(msg),
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
    }
  }
}

#[cfg(feature = "tonic")]
impl From<DataError> for tonic::Status {
  fn from(value: DataError) -> Self {
    match value {
      DataError::BizError { code, msg } => make_tonic_status(code, msg),
      DataError::InternalError { code, msg, .. } => make_tonic_status(code, msg),
      DataError::SecurityError(_) => tonic::Status::unauthenticated("Token error"),
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

#[cfg(feature = "modelsql")]
impl From<modelsql::SqlError> for DataError {
  fn from(value: modelsql::SqlError) -> Self {
    DataError::server_error(value.to_string())
  }
}

#[cfg(feature = "modelsql")]
impl From<modelsql::store::DbxError> for DataError {
  fn from(value: modelsql::store::DbxError) -> Self {
    DataError::server_error(value.to_string())
  }
}
