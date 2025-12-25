//! # Fusion AI Error Types
//!
//! This module provides error types for the fusion-ai crate.
//! For rig 0.27+, errors from the factory module use [`factory::FactoryError`].

pub use crate::factory::FactoryError;
use rig::completion::CompletionError;
use rig::image_generation::ImageGenerationError;

/// AI-related errors
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AiError {
  #[error("Custom error: {0}")]
  Custom(String),

  #[error(transparent)]
  FactoryError(#[from] FactoryError),

  #[error(transparent)]
  CompletionError(#[from] CompletionError),

  #[error(transparent)]
  ImageGenerationError(#[from] ImageGenerationError),
}
