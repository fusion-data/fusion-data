use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use serde_json::Value;
use ulid::Ulid;
use ultimate::security;
use ultimate::DataError;

pub type AppResult<T> = core::result::Result<Json<T>, AppError>;

/// A default error response for most API errors.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AppError {
  /// A unique error ID.
  pub err_id: Ulid,

  pub err_code: i32,

  /// An error message.
  pub err_msg: String,

  /// Optional Additional error details.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub err_msg_detail: Option<Value>,
}

impl AppError {
  pub fn new(error: impl Into<String>) -> Self {
    Self { err_id: Ulid::new(), err_code: 500, err_msg: error.into(), err_msg_detail: None }
  }

  pub fn new_with_code(err_code: i32, err_msg: impl Into<String>) -> Self {
    Self { err_id: Ulid::new(), err_code, err_msg: err_msg.into(), err_msg_detail: None }
  }

  pub fn with_err_code(mut self, err_code: i32) -> Self {
    self.err_code = err_code;
    self
  }

  pub fn with_details(mut self, details: Value) -> Self {
    if details == Value::Null {
      self.err_msg_detail = None
    } else {
      self.err_msg_detail = Some(details);
    }
    self
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> axum::response::Response {
    let status = StatusCode::from_u16(self.err_code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut res = axum::Json(self).into_response();
    *res.status_mut() = status;
    res
  }
}

impl From<DataError> for AppError {
  fn from(err: DataError) -> Self {
    match err {
      DataError::BizError { code, msg } => Self::new(msg).with_err_code(code),
      DataError::SecurityError(e) => convert_security_error(e),
      DataError::UltimateCommonError(e) => Self::new(e.to_string()),
      DataError::SystemTimeError(e) => Self::new(e.to_string()),
      DataError::ParseIntError(e) => Self::new(e.to_string()),
      DataError::IoError(e) => Self::new(e.to_string()),
      DataError::JsonError(e) => Self::new(e.to_string()),
      DataError::TaskJoinError(e) => Self::new(e.to_string()),
    }
  }
}

impl From<hyper::Error> for AppError {
  fn from(value: hyper::Error) -> Self {
    AppError::new_with_code(500, value.to_string())
  }
}

impl From<serde_json::Error> for AppError {
  fn from(value: serde_json::Error) -> Self {
    AppError::new_with_code(500, value.to_string())
  }
}

fn convert_security_error(e: security::Error) -> AppError {
  // match e {
  //     security::Error::HmacFailNewFromSlice => todo!(),
  //     security::Error::InvalidFormat => todo!(),
  //     security::Error::CannotDecodeIdent => todo!(),
  //     security::Error::CannotDecodeExp => todo!(),
  //     security::Error::SignatureNotMatching => todo!(),
  //     security::Error::ExpNotIso => todo!(),
  //     security::Error::Expired => todo!(),
  //     security::Error::JoseError(_) => todo!(),
  //     security::Error::FailedToHashPassword => todo!(),
  //     security::Error::InvalidPassword => todo!(),
  //     security::Error::FailedToVerifyPassword => todo!(),
  // }
  AppError::new(e.to_string())
}
