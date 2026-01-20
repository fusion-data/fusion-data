use derive_builder::Builder;
use rig::{
  completion::{CompletionRequestBuilder, CompletionResponse},
  message::Message,
  streaming::StreamingCompletionResponse,
};
use serde::{Deserialize, Serialize};

use crate::factory::{AgentConfig as FactoryAgentConfig, ClientFactory, FactoryError};

/// Agent configuration for creating agents.
/// This is a simpler config that uses the factory pattern internally.
///
/// For more control, use [`factory::AgentConfig`] directly.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
  #[builder(setter(into))]
  pub provider: String,

  #[builder(setter(into))]
  pub model: String,

  /// Optional API base URL for the model provider
  #[builder(default, setter(into, strip_option))]
  pub base_url: Option<String>,
  /// Optional API key for the model provider
  #[builder(default, setter(into, strip_option))]
  pub api_key: Option<String>,

  /// Name of the agent used for logging and debugging
  #[builder(default, setter(into, strip_option))]
  pub name: Option<String>,
  /// Agent description. Primarily useful when using sub-agents as part of an agent workflow and converting agents to other formats.
  #[builder(default, setter(into, strip_option))]
  pub description: Option<String>,
  /// System prompt
  #[builder(default, setter(into, strip_option))]
  pub system_prompt: Option<String>,
  /// Context documents always available to the agent
  #[builder(default, setter(into))]
  pub static_context: Vec<String>,
  /// Tools that are always available to the agent (by name)
  #[builder(default, setter(into))]
  pub static_tools: Vec<String>,
  /// Maximum number of tokens for the completion
  #[builder(default, setter(strip_option))]
  pub max_tokens: Option<u64>,
  /// Temperature of the model
  #[builder(default, setter(strip_option))]
  pub temperature: Option<f64>,
  /// Additional parameters to be passed to the model
  #[builder(default, setter(into, strip_option))]
  pub additional_params: Option<serde_json::Value>,
}

impl From<AgentConfig> for FactoryAgentConfig {
  fn from(config: AgentConfig) -> Self {
    FactoryAgentConfig {
      provider: config.provider,
      model: config.model,
      base_url: config.base_url,
      api_key: config.api_key,
      name: config.name,
      description: config.description,
      system_prompt: config.system_prompt,
      static_context: config.static_context,
      max_tokens: config.max_tokens,
      temperature: config.temperature,
      additional_params: config.additional_params,
    }
  }
}

/// A model agent that can invoke completions with a specific configuration.
/// The generic parameter `M` represents the underlying completion model type.
#[derive(Clone)]
pub struct ModelAgent<M: rig::completion::CompletionModel> {
  config: AgentConfig,
  _model: std::marker::PhantomData<M>,
}

impl<M: rig::completion::CompletionModel> ModelAgent<M> {
  /// Create a new ModelAgent from the given configuration.
  pub fn new(config: AgentConfig) -> Self {
    Self { config, _model: std::marker::PhantomData }
  }

  /// Invoke the agent with the given prompt and chat history, returning the full completion response.
  pub async fn invoke(&self, prompt: &str, chat_history: Vec<Message>) -> Result<CompletionResponse<M::Response>, FactoryError> {
    let factory = ClientFactory::new();
    let agent = factory.agent(&self.config.clone().into())?;
    let request = agent.completion(prompt, chat_history).await?;
    let response = request.send().await?;
    Ok(response)
  }

  /// Create a completion request builder for the given prompt and chat history.
  pub async fn completion(
    &self,
    prompt: &str,
    chat_history: Vec<Message>,
  ) -> Result<CompletionRequestBuilder<M>, FactoryError> {
    let factory = ClientFactory::new();
    let agent = factory.agent(&self.config.clone().into())?;
    let request = agent.completion(prompt, chat_history).await?;
    Ok(request)
  }

  /// Invoke the agent with the given prompt and chat history, returning a streaming response.
  pub async fn stream(
    &self,
    prompt: &str,
    chat_history: Vec<Message>,
  ) -> Result<StreamingCompletionResponse<M::StreamingResponse>, FactoryError> {
    let factory = ClientFactory::new();
    let agent = factory.agent(&self.config.clone().into())?;
    let request = agent.completion(prompt, chat_history).await?;
    let response = request.stream().await?;
    Ok(response)
  }
}

impl From<&AgentConfig> for ModelAgent<rig::providers::openai::CompletionModel> {
  fn from(config: &AgentConfig) -> Self {
    Self::new(config.clone())
  }
}
