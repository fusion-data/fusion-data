//! # Embeddings Module
//!
//! This module provides embedding configuration and utilities.
//! For the recommended rig 0.27+ pattern, use [`factory::EmbeddingConfig`] instead.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::factory::{ClientFactory, EmbeddingConfig as FactoryEmbeddingConfig, FactoryError};

/// Embedding configuration for creating embedding models.
/// This is a simpler config that uses the factory pattern internally.
///
/// For more control, use [`factory::EmbeddingConfig`] directly.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingConfig {
  #[builder(setter(into))]
  pub provider: String,

  #[builder(setter(into))]
  pub model: String,

  pub dims: usize,

  #[builder(default, setter(into, strip_option))]
  pub base_url: Option<String>,

  #[builder(default, setter(into, strip_option))]
  pub api_key: Option<String>,
}

impl From<EmbeddingConfig> for FactoryEmbeddingConfig {
  fn from(config: EmbeddingConfig) -> Self {
    FactoryEmbeddingConfig {
      provider: config.provider,
      model: config.model,
      dims: config.dims,
      base_url: config.base_url,
      api_key: config.api_key,
    }
  }
}

/// Embeddings wrapper that provides a simple interface for generating embeddings.
#[derive(Clone)]
pub struct Embeddings<M: rig::embeddings::EmbeddingModel> {
  config: EmbeddingConfig,
  _model: std::marker::PhantomData<M>,
}

impl<M: rig::embeddings::EmbeddingModel> Embeddings<M> {
  /// Create new Embeddings wrapper
  pub fn new(config: EmbeddingConfig) -> Self {
    Self { config, _model: std::marker::PhantomData }
  }

  /// Generate embeddings for the given documents
  pub async fn embed(&self, documents: Vec<String>) -> Result<Vec<Vec<f64>>, FactoryError> {
    let factory = ClientFactory::new();
    let config = self.config.clone().into();
    let embeddings = factory.embeddings(&config, documents).await?;
    let embeddings = embeddings.into_iter().map(|e| e.vec).collect();
    Ok(embeddings)
  }
}

impl From<&EmbeddingConfig> for Embeddings<rig::providers::openai::EmbeddingModel<reqwest::Client>> {
  fn from(config: &EmbeddingConfig) -> Self {
    Self::new(config.clone())
  }
}
