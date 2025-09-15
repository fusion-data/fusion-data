use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use fusion_core::DataError;
use serde::Serialize;
use serde_json::Value;

pub type WebResult<T> = core::result::Result<Json<T>, WebError>;

/// A default error response for most API errors.
#[derive(Debug, Serialize)]
// #[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct WebError {
  /// A unique error ID.
  // TODO 应从 tracing 中获取
  // pub err_id: Ulid,

  /// A unique error code.
  pub err_code: i32,

  /// An error message.
  pub err_msg: String,

  /// Optional Additional error details.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub err_detail: Option<Box<Value>>,
}

impl WebError {
  pub fn new(err_code: i32, err_msg: impl Into<String>, err_detail: Option<Box<Value>>) -> Self {
    Self { err_code, err_msg: err_msg.into(), err_detail }
  }

  pub fn new_with_msg(err_msg: impl Into<String>) -> Self {
    Self::new(500, err_msg, None)
  }

  pub fn new_with_code(err_code: i32, err_msg: impl Into<String>) -> Self {
    Self::new(err_code, err_msg, None)
  }

  pub fn server_error_with_detail(err_msg: impl Into<String>, err_detail: Box<Value>) -> Self {
    Self::new(500, err_msg, Some(err_detail))
  }

  pub fn with_err_code(mut self, err_code: i32) -> Self {
    self.err_code = err_code;
    self
  }

  pub fn with_details(mut self, details: Box<Value>) -> Self {
    if *details == Value::Null {
      self.err_detail = None
    } else {
      self.err_detail = Some(details);
    }
    self
  }

  pub fn with_err_msg(mut self, err_msg: impl Into<String>) -> Self {
    self.err_msg = err_msg.into();
    self
  }
}

impl IntoResponse for WebError {
  fn into_response(self) -> axum::response::Response {
    let status = StatusCode::from_u16(self.err_code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut res = axum::Json(self).into_response();
    *res.status_mut() = status;
    res
  }
}

impl From<DataError> for WebError {
  fn from(err: DataError) -> Self {
    match err {
      DataError::BizError { code, msg, detail } => {
        let error = Self::new_with_msg(msg).with_err_code(code);
        if let Some(v) = detail { error.with_details(v) } else { error }
      }
      DataError::InternalError { code, msg, .. } => Self::new_with_msg(msg).with_err_code(code),
      DataError::SystemTimeError(e) => Self::new_with_msg(e.to_string()),
      DataError::ParseIntError(e) => Self::new_with_msg(e.to_string()).with_err_code(400),
      DataError::IoError(e) => Self::new_with_msg(e.to_string()),
      DataError::JsonError(e) => Self::new_with_msg(e.to_string()),
    }
  }
}

impl From<hyper::Error> for WebError {
  fn from(value: hyper::Error) -> Self {
    WebError::new_with_code(500, value.to_string())
  }
}

impl From<serde_json::Error> for WebError {
  fn from(value: serde_json::Error) -> Self {
    WebError::new_with_code(500, value.to_string())
  }
}
