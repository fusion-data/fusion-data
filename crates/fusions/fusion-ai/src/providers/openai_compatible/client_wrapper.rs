use std::fmt::Debug;

#[cfg(feature = "audio")]
use crate::providers::openai_compatible::audio_generation::AudioGenerationModel;
use crate::providers::openai_compatible::{
  self, EmbeddingModel, ImageGenerationModel, TranscriptionModel, completion::CompletionModel,
};
use rig::client::{CompletionClient, ProviderClient, ProviderValue, builder::ClientBuildError};

use crate::agents::AgentConfig;

// ================================================================
// OpenAI-Compatible Client using Completion API
// ================================================================

pub struct ClientBuilder<'a> {
  api_key: &'a str,
  base_url: &'a str,
  http_client: reqwest::Client,
}

impl<'a> ClientBuilder<'a> {
  pub fn new(base_url: &'a str, api_key: &'a str) -> Self {
    Self { api_key, base_url, http_client: reqwest::Client::new() }
  }

  #[allow(dead_code)]
  pub fn with_client(mut self, http_client: reqwest::Client) -> Self {
    self.http_client = http_client;
    self
  }

  pub fn build(self) -> ClientWrapper {
    let inner = openai_compatible::client::Client::<reqwest::Client>::builder(self.api_key)
      .base_url(self.base_url)
      .with_client(self.http_client)
      .build();
    ClientWrapper(inner)
  }
}

#[derive(Debug, Clone)]
pub struct ClientWrapper(openai_compatible::client::Client<reqwest::Client>);

impl ClientWrapper {
  /// Create a new OpenAI-compatible client.
  pub fn new(base_url: &str, api_key: &str) -> Self {
    ClientBuilder::new(base_url, api_key).build()
  }

  /// Convert to OpenAI-compatible client for use with CompletionModel
  pub fn to_openai_client(&self) -> openai_compatible::client::Client<reqwest::Client> {
    // openai::Client::<reqwest::Client>::builder(&self.api_key).base_url(&self.base_url).build()
    self.0.clone()
  }
}

// Implement CompletionClient using completion API (not responses API)
impl CompletionClient for ClientWrapper {
  type CompletionModel = CompletionModel<reqwest::Client>;

  fn completion_model(&self, model: &str) -> Self::CompletionModel {
    let openai_client = self.to_openai_client();
    CompletionModel::new(openai_client, model)
  }
}

// Implement other required traits with not supported functionality
impl rig::client::EmbeddingsClient for ClientWrapper {
  type EmbeddingModel = EmbeddingModel<reqwest::Client>;

  fn embedding_model(&self, model: &str) -> Self::EmbeddingModel {
    self.to_openai_client().embedding_model(model)
  }

  fn embedding_model_with_ndims(&self, model: &str, ndims: usize) -> Self::EmbeddingModel {
    self.to_openai_client().embedding_model_with_ndims(model, ndims)
  }
}

impl rig::client::TranscriptionClient for ClientWrapper {
  type TranscriptionModel = TranscriptionModel<reqwest::Client>;

  fn transcription_model(&self, model: &str) -> Self::TranscriptionModel {
    self.to_openai_client().transcription_model(model)
  }
}

#[cfg(feature = "image")]
impl rig::client::ImageGenerationClient for ClientWrapper {
  type ImageGenerationModel = ImageGenerationModel<reqwest::Client>;

  fn image_generation_model(&self, model: &str) -> Self::ImageGenerationModel {
    self.to_openai_client().image_generation_model(model)
  }
}

#[cfg(feature = "audio")]
impl rig::client::AudioGenerationClient for ClientWrapper {
  type AudioGenerationModel = AudioGenerationModel<reqwest::Client>;

  fn audio_generation_model(&self, model: &str) -> Self::AudioGenerationModel {
    self.to_openai_client().audio_generation_model(model)
  }
}

impl ProviderClient for ClientWrapper {
  fn from_env() -> Self {
    let api_key = std::env::var("OPENAI_COMPATIBLE_API_KEY")
      .unwrap_or_else(|_| std::env::var("OPENAI_API_KEY").unwrap_or_default());
    Self(openai_compatible::client::Client::new(&api_key))
  }

  fn from_val(provider_value: ProviderValue) -> Self {
    let api_key = match provider_value {
      ProviderValue::Simple(key) => key,
      ProviderValue::ApiKeyWithOptionalKey(key, _) => key,
      ProviderValue::ApiKeyWithVersionAndHeader(key, _, _) => key,
    };
    Self(openai_compatible::client::Client::new(&api_key))
  }
}

pub fn create_client(config: &AgentConfig) -> Result<Box<dyn ProviderClient>, ClientBuildError> {
  if let Some(base_url) = config.base_url.as_deref()
    && let Some(api_key) = config.api_key.as_deref()
  {
    Ok(Box::new(ClientWrapper::new(base_url, api_key)))
  } else {
    Err(ClientBuildError::FactoryError("base_url or api_key".to_string()))
  }
}

pub fn func_env() -> Box<dyn ProviderClient> {
  Box::new(ClientWrapper::from_env())
}

pub fn func_val(provider_value: ProviderValue) -> Box<dyn ProviderClient> {
  Box::new(ClientWrapper::from_val(provider_value))
}
