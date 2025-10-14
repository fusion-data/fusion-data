use serde::{Deserialize, Serialize};
use std::fmt;

/// Sdk Result type
pub type SdkResult<T> = Result<T, SdkError>;

/// SDK error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SdkError {
  /// HTTP request error
  HttpError(String),
  /// JSON serialization/deserialization error
  JsonError(String),
  /// Configuration error
  ConfigError(String),
  /// API error response
  ApiError {
    /// HTTP status code
    status: u16,
    /// Error message
    message: String,
  },
  /// Network error
  NetworkError(String),
  /// Authentication error
  AuthError(String),
  /// Validation error
  ValidationError(String),
  /// Unknown error
  Unknown(String),
}

impl fmt::Display for SdkError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SdkError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
      SdkError::JsonError(msg) => write!(f, "JSON error: {}", msg),
      SdkError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
      SdkError::ApiError { status, message } => {
        write!(f, "API error ({}): {}", status, message)
      }
      SdkError::NetworkError(msg) => write!(f, "Network error: {}", msg),
      SdkError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
      SdkError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
      SdkError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
    }
  }
}

impl std::error::Error for SdkError {}

#[cfg(not(target_arch = "wasm32"))]
impl From<reqwest::Error> for SdkError {
  fn from(error: reqwest::Error) -> Self {
    if error.is_timeout() {
      SdkError::NetworkError("Request timeout".to_string())
    } else if error.is_connect() {
      SdkError::NetworkError("Connection error".to_string())
    } else if let Some(status) = error.status() {
      SdkError::ApiError { status: status.as_u16(), message: error.to_string() }
    } else {
      SdkError::HttpError(error.to_string())
    }
  }
}

#[cfg(target_arch = "wasm32")]
impl From<gloo_net::Error> for SdkError {
  fn from(error: gloo_net::Error) -> Self {
    SdkError::NetworkError(error.to_string())
  }
}

impl From<serde_json::Error> for SdkError {
  fn from(error: serde_json::Error) -> Self {
    SdkError::JsonError(error.to_string())
  }
}
