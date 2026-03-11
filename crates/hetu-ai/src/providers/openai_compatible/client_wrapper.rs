use std::fmt::Debug;

use rig::client::CompletionClient;
use rig::completion::{CompletionError, CompletionRequest, CompletionResponse, GetTokenUsage};
use rig::streaming::StreamingCompletionResponse;

use super::client::Client as OpenAIClient;
use super::completion::{CompletionModel, CompletionResponse as InnerCompletionResponse};
use super::streaming::StreamingCompletionResponse as InnerStreamingCompletionResponse;

use crate::FactoryError;
use crate::agents::AgentConfig;

// ================================================================
// OpenAI-Compatible Client Completion Model for ClientWrapper
// ================================================================

/// CompletionModel wrapper for ClientWrapper that implements CompletionModel trait
#[derive(Clone, Debug)]
pub struct ClientWrapperCompletionModel {
  client: ClientWrapper,
  model: String,
}

impl ClientWrapperCompletionModel {
  pub fn new(client: ClientWrapper, model: impl Into<String>) -> Self {
    Self { client, model: model.into() }
  }

  pub fn into_agent_builder(self) -> rig::agent::AgentBuilder<Self> {
    rig::agent::AgentBuilder::new(self)
  }
}

impl rig::completion::CompletionModel for ClientWrapperCompletionModel {
  type Response = InnerCompletionResponse;
  type StreamingResponse = InnerStreamingCompletionResponse;
  type Client = ClientWrapper;

  fn make(client: &Self::Client, model: impl Into<String>) -> Self {
    Self::new(client.clone(), model.into())
  }

  #[cfg_attr(feature = "worker", worker::send)]
  async fn completion(
    &self,
    request: CompletionRequest,
  ) -> Result<CompletionResponse<Self::Response>, CompletionError> {
    self.client.completion_model(&self.model).completion(request).await
  }

  #[cfg_attr(feature = "worker", worker::send)]
  async fn stream(
    &self,
    request: CompletionRequest,
  ) -> Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError> {
    self.client.completion_model(&self.model).stream(request).await
  }
}

impl GetTokenUsage for InnerCompletionResponse {
  fn token_usage(&self) -> Option<rig::completion::Usage> {
    self.usage.as_ref().map(|usage| {
      let mut token_usage = rig::completion::Usage::new();
      token_usage.input_tokens = usage.prompt_tokens as u64;
      token_usage.output_tokens = (usage.total_tokens - usage.prompt_tokens) as u64;
      token_usage.total_tokens = usage.total_tokens as u64;
      token_usage
    })
  }
}

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
    let inner = OpenAIClient::<reqwest::Client>::builder(self.api_key)
      .base_url(self.base_url)
      .with_client(self.http_client)
      .build();
    ClientWrapper(inner)
  }
}

#[derive(Debug, Clone)]
pub struct ClientWrapper(OpenAIClient<reqwest::Client>);

impl ClientWrapper {
  /// Create a new OpenAI-compatible client.
  pub fn new(base_url: &str, api_key: &str) -> Self {
    ClientBuilder::new(base_url, api_key).build()
  }

  /// Get the inner OpenAI-compatible client
  pub fn to_inner(&self) -> &OpenAIClient<reqwest::Client> {
    &self.0
  }

  /// Get the inner OpenAI-compatible client (cloned)
  pub fn to_inner_cloned(&self) -> OpenAIClient<reqwest::Client> {
    self.0.clone()
  }

  /// Create a completion model
  pub fn completion_model(&self, model: &str) -> CompletionModel<reqwest::Client> {
    CompletionModel::new(self.0.clone(), model)
  }
}

/// Create an OpenAI-compatible client from config
pub fn create_client(config: &AgentConfig) -> Result<ClientWrapper, FactoryError> {
  let base_url = config
    .base_url
    .as_ref()
    .ok_or_else(|| FactoryError::MissingBaseUrl("base_url is required for openai-compatible provider".to_string()))?;
  let api_key = config
    .api_key
    .as_ref()
    .ok_or_else(|| FactoryError::MissingApiKey("api_key is required for openai-compatible provider".to_string()))?;
  Ok(ClientWrapper::new(base_url, api_key))
}

impl CompletionClient for ClientWrapper {
  type CompletionModel = ClientWrapperCompletionModel;

  fn completion_model(&self, model: impl Into<String>) -> Self::CompletionModel {
    ClientWrapperCompletionModel::new(self.clone(), model.into())
  }
}
