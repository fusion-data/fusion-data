use derive_builder::Builder;
use serde::{Deserialize, Serialize};

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
