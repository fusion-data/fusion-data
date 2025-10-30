use derive_builder::Builder;
use rig::{
  client::{builder::FinalCompletionResponse, completion::CompletionModelHandle},
  completion::{Completion, CompletionRequestBuilder, CompletionResponse},
  message::Message,
  streaming::StreamingCompletionResponse,
};
use serde::{Deserialize, Serialize};

use crate::{agents::AgentError, factory::ClientBuilderFactory};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Builder)]
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
  // /// List of vector store, with the sample number
  // dynamic_context: Vec<(usize, Box<dyn VectorStoreIndexDyn>)>,
  // /// Dynamic tools
  // dynamic_tools: Vec<(usize, Box<dyn VectorStoreIndexDyn>)>,

  // /// Actual tool implementations
  // tools: ToolSet,
  // /// Whether or not the underlying LLM should be forced to use a tool before providing a response.
  // #[builder(default, setter(strip_option))]
  // pub tool_choice: Option<ToolChoice>,
}

#[derive(Clone)]
pub struct ModelAgent {
  pub config: AgentConfig,
}

impl ModelAgent {
  /// Create a new ModelAgent from the given configuration.
  pub fn new(config: AgentConfig) -> Self {
    Self { config }
  }

  /// Invoke the agent with the given prompt and chat history, returning the full completion response.
  pub async fn invoke(&self, prompt: &str, chat_history: Vec<Message>) -> Result<CompletionResponse<()>, AgentError> {
    let request = self.completion(prompt, chat_history).await?;
    let response: CompletionResponse<()> = request.send().await?;
    Ok(response)
  }

  /// Create a completion request builder for the given prompt and chat history.
  async fn completion(
    &self,
    prompt: &str,
    chat_history: Vec<Message>,
  ) -> Result<CompletionRequestBuilder<CompletionModelHandle<'_>>, AgentError> {
    let factory = ClientBuilderFactory::new();
    let agent = factory.agent(&self.config)?;
    let request = agent.completion(prompt, chat_history).await?;
    Ok(request)
  }

  /// Invoke the agent with the given prompt and chat history, returning a streaming response.
  pub async fn stream(
    &self,
    prompt: &str,
    chat_history: Vec<Message>,
  ) -> Result<StreamingCompletionResponse<FinalCompletionResponse>, AgentError> {
    let factory = ClientBuilderFactory::new();
    let agent = factory.agent(&self.config)?;
    let request = agent.completion(prompt, chat_history).await?;
    let response = request.stream().await?;
    Ok(response)
  }
}

impl From<&AgentConfig> for ModelAgent {
  fn from(config: &AgentConfig) -> Self {
    Self::new(config.clone())
  }
}
