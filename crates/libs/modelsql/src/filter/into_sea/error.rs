pub type SeaResult<T> = core::result::Result<T, IntoSeaError>;

/// Error for FilterNode to Sea Condition
#[derive(Debug, thiserror::Error)]
pub enum IntoSeaError {
  // For now, just Custom. Might have more variants later.
  #[error("Custom error: {0}")]
  Custom(String),

  #[error(transparent)]
  SerdeJson(#[from] serde_json::Error),
}

impl IntoSeaError {
  pub fn custom(message: impl Into<String>) -> Self {
    IntoSeaError::Custom(message.into())
  }
}
