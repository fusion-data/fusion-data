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

  // 4xx Client Errors
  pub fn bad_request(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(400, err_msg)
  }

  pub fn unauthorized(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(401, err_msg)
  }

  pub fn payment_required(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(402, err_msg)
  }

  pub fn forbidden(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(403, err_msg)
  }

  pub fn not_found(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(404, err_msg)
  }

  pub fn method_not_allowed(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(405, err_msg)
  }

  pub fn not_acceptable(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(406, err_msg)
  }

  pub fn proxy_authentication_required(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(407, err_msg)
  }

  pub fn request_timeout(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(408, err_msg)
  }

  pub fn conflict(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(409, err_msg)
  }

  pub fn gone(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(410, err_msg)
  }

  pub fn length_required(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(411, err_msg)
  }

  pub fn precondition_failed(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(412, err_msg)
  }

  pub fn payload_too_large(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(413, err_msg)
  }

  pub fn uri_too_long(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(414, err_msg)
  }

  pub fn unsupported_media_type(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(415, err_msg)
  }

  pub fn range_not_satisfiable(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(416, err_msg)
  }

  pub fn expectation_failed(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(417, err_msg)
  }

  pub fn misdirected_request(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(421, err_msg)
  }

  pub fn unprocessable_entity(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(422, err_msg)
  }

  pub fn locked(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(423, err_msg)
  }

  pub fn failed_dependency(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(424, err_msg)
  }

  pub fn too_early(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(425, err_msg)
  }

  pub fn upgrade_required(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(426, err_msg)
  }

  pub fn precondition_required(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(428, err_msg)
  }

  pub fn too_many_requests(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(429, err_msg)
  }

  pub fn request_header_fields_too_large(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(431, err_msg)
  }

  pub fn unavailable_for_legal_reasons(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(451, err_msg)
  }

  // 5xx Server Errors
  pub fn server_error(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(500, err_msg)
  }

  pub fn not_implemented(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(501, err_msg)
  }

  pub fn bad_gateway(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(502, err_msg)
  }

  pub fn service_unavailable(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(503, err_msg)
  }

  pub fn gateway_timeout(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(504, err_msg)
  }

  pub fn http_version_not_supported(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(505, err_msg)
  }

  pub fn variant_also_negotiates(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(506, err_msg)
  }

  pub fn insufficient_storage(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(507, err_msg)
  }

  pub fn loop_detected(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(508, err_msg)
  }

  pub fn not_extended(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(510, err_msg)
  }

  pub fn network_authentication_required(err_msg: impl Into<String>) -> Self {
    Self::new_with_code(511, err_msg)
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

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::StatusCode;
  use axum::response::IntoResponse;

  #[test]
  fn test_http_error_functions() {
    // Test 4xx client errors
    let error = WebError::bad_request("Invalid request");
    assert_eq!(error.err_code, 400);
    assert_eq!(error.err_msg, "Invalid request");

    let error = WebError::unauthorized("Unauthorized access");
    assert_eq!(error.err_code, 401);
    assert_eq!(error.err_msg, "Unauthorized access");

    let error = WebError::forbidden("Access forbidden");
    assert_eq!(error.err_code, 403);
    assert_eq!(error.err_msg, "Access forbidden");

    let error = WebError::not_found("Resource not found");
    assert_eq!(error.err_code, 404);
    assert_eq!(error.err_msg, "Resource not found");

    let error = WebError::method_not_allowed("Method not allowed");
    assert_eq!(error.err_code, 405);
    assert_eq!(error.err_msg, "Method not allowed");

    let error = WebError::conflict("Resource conflict");
    assert_eq!(error.err_code, 409);
    assert_eq!(error.err_msg, "Resource conflict");

    let error = WebError::unprocessable_entity("Unprocessable entity");
    assert_eq!(error.err_code, 422);
    assert_eq!(error.err_msg, "Unprocessable entity");

    let error = WebError::too_many_requests("Rate limit exceeded");
    assert_eq!(error.err_code, 429);
    assert_eq!(error.err_msg, "Rate limit exceeded");

    // Test 5xx server errors
    let error = WebError::server_error("Internal server error");
    assert_eq!(error.err_code, 500);
    assert_eq!(error.err_msg, "Internal server error");

    let error = WebError::not_implemented("Feature not implemented");
    assert_eq!(error.err_code, 501);
    assert_eq!(error.err_msg, "Feature not implemented");

    let error = WebError::bad_gateway("Bad gateway");
    assert_eq!(error.err_code, 502);
    assert_eq!(error.err_msg, "Bad gateway");

    let error = WebError::service_unavailable("Service unavailable");
    assert_eq!(error.err_code, 503);
    assert_eq!(error.err_msg, "Service unavailable");

    let error = WebError::gateway_timeout("Gateway timeout");
    assert_eq!(error.err_code, 504);
    assert_eq!(error.err_msg, "Gateway timeout");
  }

  #[test]
  fn test_into_response() {
    let error = WebError::not_found("Resource not found");
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error = WebError::server_error("Server error");
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let error = WebError::bad_request("Invalid input");
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  }
}
