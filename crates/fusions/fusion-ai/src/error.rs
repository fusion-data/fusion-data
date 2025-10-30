use rig::client::builder::ClientBuildError;
use rig::completion::CompletionError;
use rig::image_generation::ImageGenerationError;

#[derive(Debug, thiserror::Error)]
pub enum AiError {
  #[error("Custom error: {0}")]
  Custom(String),

  #[error(transparent)]
  ClientBuilderError(#[from] ClientBuildError),

  #[error(transparent)]
  CompletionError(#[from] CompletionError),

  #[error(transparent)]
  ImageGenerationError(#[from] ImageGenerationError),
}
