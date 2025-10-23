use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use fusion_core::DataError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type WebResult<T> = core::result::Result<Json<T>, WebError>;

/// A default error response for most API errors.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "with-openapi",
  derive(utoipa::ToSchema, utoipa::ToResponse),
  response(description = "A default error response for most API errors.")
)]
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
  pub detail: Option<Value>,
}

impl WebError {
  pub fn new(err_code: i32, err_msg: impl Into<String>, detail: Option<Value>) -> Self {
    Self { err_code, err_msg: err_msg.into(), detail }
  }

  pub fn new_with_msg(err_msg: impl Into<String>) -> Self {
    Self::new(500, err_msg, None)
  }

  pub fn new_with_code(err_code: i32, err_msg: impl Into<String>) -> Self {
    Self::new(err_code, err_msg, None)
  }

  pub fn server_error_with_detail(err_msg: impl Into<String>, detail: Value) -> Self {
    Self::new(500, err_msg, Some(detail))
  }

  pub fn with_err_code(mut self, err_code: i32) -> Self {
    self.err_code = err_code;
    self
  }

  pub fn with_details(mut self, details: Value) -> Self {
    if details == Value::Null {
      self.detail = None
    } else {
      self.detail = Some(details);
    }
    self
  }

  pub fn with_err_msg(mut self, err_msg: impl Into<String>) -> Self {
    self.err_msg = err_msg.into();
    self
  }

  /// Create a 401 Unauthorized error
  pub fn unauthorized(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(401, err_msg)
  }

  /// Create a 403 Forbidden error
  pub fn forbidden(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(403, err_msg)
  }

  /// Create a 400 Bad Request error
  pub fn bad_request(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(400, err_msg)
  }

  /// Create a 502 Bad Gateway error
  pub fn bad_gateway(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(502, err_msg)
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
    // Log the source error if present
    if let Some(source) = err.source.as_ref() {
      log::error!("DataError with code {}, msg {} has source: {:?}", err.code, err.msg, source);
    }

    let mut web_error = Self::new_with_msg(err.msg.clone()).with_err_code(err.code);

    // Add details if present
    if let Some(data) = err.data.as_ref() {
      web_error = web_error.with_details(data.clone());
    }

    web_error
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
