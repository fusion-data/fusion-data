//! # Rig Provider Client Factory
//!
//! This module provides a unified factory for creating LLM provider clients
//! using the explicit provider pattern (recommended for rig 0.27+).
//!
//! ## Example
//!
//! ```
//! use ultimate_ai::factory::ClientFactory;
//!
//! let factory = ClientFactory::new();
//! let openai_client = factory.openai("sk-...")?;
//! let deepseek_client = factory.deepseek("sk-...", Some("https://api.deepseek.com"))?;
//! ```

use derive_builder::Builder;
use rig::client::{CompletionClient, EmbeddingsClient};
use rig::embeddings::{Embedding, EmbeddingModel};
use rig::{agent::Agent, embeddings, http_client};
use thiserror::Error;

use crate::providers::openai_compatible::{ClientWrapper, ClientWrapperCompletionModel, create_client};

use rig::providers::{
  anthropic, anthropic::completion::CompletionModel as AnthropicCompletionModel, cohere,
  cohere::CompletionModel as CohereCompletionModel, deepseek, deepseek::CompletionModel as DeepSeekCompletionModel,
  gemini, gemini::completion::CompletionModel as GeminiCompletionModel, groq,
  groq::CompletionModel as GroqCompletionModel, huggingface,
  huggingface::completion::CompletionModel as HuggingFaceCompletionModel, mistral,
  mistral::CompletionModel as MistralCompletionModel, ollama, ollama::CompletionModel as OllamaCompletionModel, openai,
  openai::CompletionModel as OpenAICompletionModel, openrouter,
  openrouter::CompletionModel as OpenRouterCompletionModel, perplexity,
  perplexity::CompletionModel as PerplexityCompletionModel, together,
  together::CompletionModel as TogetherCompletionModel, xai, xai::completion::CompletionModel as XAICompletionModel,
};

/// Factory errors
#[derive(Debug, Error)]
pub enum FactoryError {
  #[error("Invalid provider: {0}")]
  InvalidProvider(String),

  #[error("Missing API key for provider: {0}")]
  MissingApiKey(String),

  #[error("Missing base URL for provider: {0}")]
  MissingBaseUrl(String),

  #[error("HTTP client error: {0}")]
  HttpClientError(#[from] http_client::Error),

  #[error("Embedding error: {0}")]
  EmbeddingError(#[from] embeddings::EmbeddingError),
}

/// Unified client factory for creating provider-specific clients.
/// This follows rig 0.27's recommendation of using explicit provider types
/// instead of the deprecated DynClientBuilder pattern.
#[derive(Clone, Default)]
pub struct ClientFactory {}

impl ClientFactory {
  /// Create a new client factory
  pub fn new() -> Self {
    Self {}
  }

  /// Create an OpenAI client
  pub fn openai(&self, api_key: &str) -> http_client::Result<openai::Client> {
    openai::Client::new(api_key)
  }

  /// Create an Anthropic client
  pub fn anthropic(&self, api_key: &str) -> http_client::Result<anthropic::Client> {
    anthropic::Client::new(api_key)
  }

  /// Create a DeepSeek client
  pub fn deepseek(&self, api_key: &str, base_url: Option<&str>) -> http_client::Result<deepseek::Client> {
    if let Some(url) = base_url {
      deepseek::Client::builder().api_key(api_key).base_url(url).build()
    } else {
      deepseek::Client::new(api_key)
    }
  }

  /// Create a Groq client
  pub fn groq(&self, api_key: &str) -> http_client::Result<groq::Client> {
    groq::Client::new(api_key)
  }

  /// Create an xAI client
  pub fn xai(&self, api_key: &str, base_url: Option<&str>) -> http_client::Result<xai::Client> {
    if let Some(url) = base_url {
      xai::Client::builder().api_key(api_key).base_url(url).build()
    } else {
      xai::Client::new(api_key)
    }
  }

  /// Create an OpenRouter client
  pub fn openrouter(&self, api_key: &str) -> http_client::Result<openrouter::Client> {
    openrouter::Client::new(api_key)
  }

  /// Create a Google/Gemini client
  pub fn google(&self, api_key: &str) -> http_client::Result<gemini::Client> {
    gemini::Client::new(api_key)
  }

  /// Create a Cohere client
  pub fn cohere(&self, api_key: &str) -> http_client::Result<cohere::Client> {
    cohere::Client::new(api_key)
  }

  /// Create a HuggingFace client
  pub fn huggingface(&self, api_key: &str) -> http_client::Result<huggingface::Client> {
    huggingface::Client::new(api_key)
  }

  /// Create a TogetherAI client
  pub fn togetherai(&self, api_key: &str) -> http_client::Result<together::Client> {
    together::Client::new(api_key)
  }

  /// Create a Perplexity client
  pub fn perplexity(&self, api_key: &str) -> http_client::Result<perplexity::Client> {
    perplexity::Client::new(api_key)
  }

  /// Create a Mistral client
  pub fn mistral(&self, api_key: &str) -> http_client::Result<mistral::Client> {
    mistral::Client::new(api_key)
  }

  /// Create an Ollama client (local)
  pub fn ollama(&self, base_url: &str) -> http_client::Result<ollama::Client> {
    ollama::Client::builder().base_url(base_url).api_key(rig::client::Nothing).build()
  }

  /// Create an OpenAI-compatible client
  pub fn openai_compatible(&self, base_url: &str, api_key: &str) -> ClientWrapper {
    ClientWrapper::new(base_url, api_key)
  }

  /// Create an OpenAI agent
  pub fn openai_agent(
    &self,
    config: &AgentConfig,
    client: &openai::Client,
  ) -> Result<Agent<OpenAICompletionModel>, FactoryError> {
    // completions_api() takes ownership, so we clone the client
    let completions_client = client.clone().completions_api();
    let mut builder = completions_client.agent(&config.model);
    if let Some(system_prompt) = &config.system_prompt {
      builder = builder.preamble(system_prompt);
    }
    for doc in &config.static_context {
      builder = builder.context(doc);
    }
    if let Some(temperature) = config.temperature {
      builder = builder.temperature(temperature);
    }
    if let Some(max_tokens) = config.max_tokens {
      builder = builder.max_tokens(max_tokens);
    }
    if let Some(params) = &config.additional_params {
      builder = builder.additional_params(params.clone());
    }
    Ok(builder.build())
  }

  /// Create an Anthropic agent
  pub fn anthropic_agent(
    &self,
    config: &AgentConfig,
    client: &anthropic::Client,
  ) -> Result<Agent<AnthropicCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create a DeepSeek agent
  pub fn deepseek_agent(
    &self,
    config: &AgentConfig,
    client: &deepseek::Client,
  ) -> Result<Agent<DeepSeekCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create a Groq agent
  pub fn groq_agent(
    &self,
    config: &AgentConfig,
    client: &groq::Client,
  ) -> Result<Agent<GroqCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create an xAI agent
  pub fn xai_agent(
    &self,
    config: &AgentConfig,
    client: &xai::Client,
  ) -> Result<Agent<XAICompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create an OpenRouter agent
  pub fn openrouter_agent(
    &self,
    config: &AgentConfig,
    client: &openrouter::Client,
  ) -> Result<Agent<OpenRouterCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create a Google/Gemini agent
  pub fn google_agent(
    &self,
    config: &AgentConfig,
    client: &gemini::Client,
  ) -> Result<Agent<GeminiCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create a Cohere agent
  pub fn cohere_agent(
    &self,
    config: &AgentConfig,
    client: &cohere::Client,
  ) -> Result<Agent<CohereCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create a HuggingFace agent
  pub fn huggingface_agent(
    &self,
    config: &AgentConfig,
    client: &huggingface::Client,
  ) -> Result<Agent<HuggingFaceCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create a TogetherAI agent
  pub fn togetherai_agent(
    &self,
    config: &AgentConfig,
    client: &together::Client,
  ) -> Result<Agent<TogetherCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create a Perplexity agent
  pub fn perplexity_agent(
    &self,
    config: &AgentConfig,
    client: &perplexity::Client,
  ) -> Result<Agent<PerplexityCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create a Mistral agent
  pub fn mistral_agent(
    &self,
    config: &AgentConfig,
    client: &mistral::Client,
  ) -> Result<Agent<MistralCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create an Ollama agent
  pub fn ollama_agent(
    &self,
    config: &AgentConfig,
    client: &ollama::Client,
  ) -> Result<Agent<OllamaCompletionModel>, FactoryError> {
    self._build_agent(config, client.agent(&config.model))
  }

  /// Create an OpenAI-compatible agent
  pub fn openai_compatible_agent(
    &self,
    config: &AgentConfig,
  ) -> Result<Agent<ClientWrapperCompletionModel>, FactoryError> {
    let client = create_client(config)?;
    let mut builder = ClientWrapperCompletionModel::new(client, &config.model).into_agent_builder();
    if let Some(system_prompt) = &config.system_prompt {
      builder = builder.preamble(system_prompt);
    }
    for doc in &config.static_context {
      builder = builder.context(doc);
    }
    if let Some(temperature) = config.temperature {
      builder = builder.temperature(temperature);
    }
    Ok(builder.build())
  }

  /// Internal helper to build an agent with common options
  fn _build_agent<M: rig::completion::CompletionModel>(
    &self,
    config: &AgentConfig,
    mut builder: rig::agent::AgentBuilder<M>,
  ) -> Result<Agent<M>, FactoryError> {
    if let Some(system_prompt) = &config.system_prompt {
      builder = builder.preamble(system_prompt);
    }
    for doc in &config.static_context {
      builder = builder.context(doc);
    }
    if let Some(temperature) = config.temperature {
      builder = builder.temperature(temperature);
    }
    if let Some(max_tokens) = config.max_tokens {
      builder = builder.max_tokens(max_tokens);
    }
    if let Some(params) = &config.additional_params {
      builder = builder.additional_params(params.clone());
    }
    Ok(builder.build())
  }
}

/// Enum containing all supported provider clients
#[allow(missing_docs)]
pub enum ProviderClientEnum {
  OpenAI(Result<openai::Client, http_client::Error>),
  Anthropic(Result<anthropic::Client, http_client::Error>),
  DeepSeek(Result<deepseek::Client, http_client::Error>),
  Groq(Result<groq::Client, http_client::Error>),
  XAI(Result<xai::Client, http_client::Error>),
  OpenRouter(Result<openrouter::Client, http_client::Error>),
  Google(Result<gemini::Client, http_client::Error>),
  Cohere(Result<cohere::Client, http_client::Error>),
  HuggingFace(Result<huggingface::Client, http_client::Error>),
  TogetherAI(Result<together::Client, http_client::Error>),
  Perplexity(Result<perplexity::Client, http_client::Error>),
  Mistral(Result<mistral::Client, http_client::Error>),
  Ollama(Result<ollama::Client, http_client::Error>),
  OpenAICompatible(ClientWrapper),
}

/// Provider configuration for creating clients
#[derive(Clone, Debug, Default, Builder)]
pub struct ProviderConfig {
  #[builder(setter(into))]
  pub provider: String,

  #[builder(default, setter(into, strip_option))]
  pub base_url: Option<String>,

  #[builder(default, setter(into, strip_option))]
  pub api_key: Option<String>,
}

impl ProviderConfig {
  /// Create a new provider config
  pub fn new(provider: impl Into<String>) -> Self {
    Self { provider: provider.into(), base_url: None, api_key: None }
  }

  /// Set the API key
  pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
    self.api_key = Some(api_key.into());
    self
  }

  /// Set the base URL
  pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
    self.base_url = Some(base_url.into());
    self
  }
}

/// Agent configuration for creating agents
#[derive(Clone, Debug, Default, Builder)]
pub struct AgentConfig {
  #[builder(setter(into))]
  pub provider: String,

  #[builder(setter(into))]
  pub model: String,

  #[builder(default, setter(into, strip_option))]
  pub base_url: Option<String>,

  #[builder(default, setter(into, strip_option))]
  pub api_key: Option<String>,

  #[builder(default, setter(into, strip_option))]
  pub name: Option<String>,

  #[builder(default, setter(into, strip_option))]
  pub description: Option<String>,

  #[builder(default, setter(into, strip_option))]
  pub system_prompt: Option<String>,

  #[builder(default, setter(into))]
  pub static_context: Vec<String>,

  #[builder(default, setter(strip_option))]
  pub max_tokens: Option<u64>,

  #[builder(default, setter(strip_option))]
  pub temperature: Option<f64>,

  #[builder(default, setter(into, strip_option))]
  pub additional_params: Option<serde_json::Value>,
}

impl AgentConfig {
  /// Create a new agent config
  pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
    Self { provider: provider.into(), model: model.into(), ..Default::default() }
  }
}

impl ClientFactory {
  /// Create embeddings from configuration
  pub async fn embeddings(
    &self,
    config: &EmbeddingConfig,
    documents: Vec<String>,
  ) -> Result<Vec<Embedding>, FactoryError> {
    match config.provider.as_str() {
      "openai" => {
        let client = self.openai(config.api_key.as_deref().unwrap_or(""))?;
        let model = client.embedding_model_with_ndims(&config.model, config.dims);
        Ok(model.embed_texts(documents).await?)
      }
      "google" | "gemini" => {
        let client = self.google(config.api_key.as_deref().unwrap_or(""))?;
        let model = client.embedding_model_with_ndims(&config.model, config.dims);
        Ok(model.embed_texts(documents).await?)
      }
      "cohere" => {
        let client = self.cohere(config.api_key.as_deref().unwrap_or(""))?;
        // Cohere uses embedding_model_with_ndims which takes input_type parameter
        let model = client.embedding_model_with_ndims(&config.model, "search_document", config.dims);
        Ok(model.embed_texts(documents).await?)
      }
      "huggingface" => Err(FactoryError::InvalidProvider("HuggingFace provider no support embeddings".to_string())),
      "together" => {
        let client = self.togetherai(config.api_key.as_deref().unwrap_or(""))?;
        let model = client.embedding_model_with_ndims(&config.model, config.dims);
        Ok(model.embed_texts(documents).await?)
      }
      "mistral" => {
        let client = self.mistral(config.api_key.as_deref().unwrap_or(""))?;
        let model = client.embedding_model_with_ndims(&config.model, config.dims);
        Ok(model.embed_texts(documents).await?)
      }
      "ollama" => {
        let client = self.ollama(config.base_url.as_deref().unwrap_or("http://localhost:11434"))?;
        let model = client.embedding_model_with_ndims(&config.model, config.dims);
        Ok(model.embed_texts(documents).await?)
      }
      "openai-compatible" => {
        let client =
          self.openai_compatible(config.base_url.as_deref().unwrap_or(""), config.api_key.as_deref().unwrap_or(""));
        let model = client.to_inner().embedding_model_with_ndims(&config.model, config.dims);
        Ok(model.embed_texts(documents).await?)
      }
      _ => Err(FactoryError::InvalidProvider(config.provider.clone())),
    }
  }
}

/// Embedding configuration for creating embedding models
#[derive(Clone, Debug, Default, Builder)]
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

impl EmbeddingConfig {
  /// Create a new embedding config
  pub fn new(provider: impl Into<String>, model: impl Into<String>, dims: usize) -> Self {
    Self { provider: provider.into(), model: model.into(), dims, base_url: None, api_key: None }
  }
}
