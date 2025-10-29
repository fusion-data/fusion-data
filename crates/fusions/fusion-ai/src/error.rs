use rig::client::builder::ClientBuildError;
use rig::completion::CompletionError;

#[derive(Debug, thiserror::Error)]
pub enum AiError {
  #[error(transparent)]
  ClientBuilderError(#[from] ClientBuildError),

  #[error(transparent)]
  CompletionError(#[from] CompletionError),
}
