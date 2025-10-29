use derive_builder::Builder;
use rig::{client::builder::ClientBuildError, embeddings::embedding::EmbeddingModelDyn};
use serde::{Deserialize, Serialize};

use crate::client::ClientBuilderFactory;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Builder)]
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

#[derive(Clone)]
pub struct Embeddings {
  pub config: EmbeddingConfig,
}

impl Embeddings {
  pub fn new(config: EmbeddingConfig) -> Self {
    Self { config }
  }

  pub fn embeddings(&self) -> Result<Box<dyn EmbeddingModelDyn>, ClientBuildError> {
    let factory = ClientBuilderFactory::new();
    let client =
      factory.client(&self.config.provider, self.config.base_url.as_deref(), self.config.api_key.as_deref())?;

    let embeddings = client
      .as_embeddings()
      .ok_or(ClientBuildError::UnsupportedFeature(self.config.provider.to_string(), "embeddings".to_owned()))?;

    Ok(embeddings.embedding_model_with_ndims(&self.config.model, self.config.dims))
  }
}
