use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
  #[error("{0}")]
  Custom(String),

  #[error("Fail to decode 16U8: {context} (actual length: {actual_length})")]
  FailToDecode16U8 { context: &'static str, actual_length: usize },

  #[error("Fail to extract time from UUID v7: {0}")]
  FailExtractTimeNoUuidV7(Uuid),

  // -- Externals
  #[error(transparent)]
  Io(std::io::Error), // as example
}

impl Error {
  pub fn custom_from_err(err: impl std::error::Error) -> Self {
    Self::Custom(err.to_string())
  }

  pub fn custom(val: impl Into<String>) -> Self {
    Self::Custom(val.into())
  }
}
