use serde::Serialize;
use thiserror::Error;

pub trait DataError: core::error::Error + core::fmt::Debug + core::fmt::Display + Serialize {
  fn code(&self) -> i32;
  fn msg(&self) -> &str;
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)>;
  fn data(&self) -> Option<&serde_json::Value>;
}

pub type Result<T> = core::result::Result<T, Error>;

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
