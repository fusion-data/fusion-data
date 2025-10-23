//! DeepSeek LLM Node Implementation
//!
//! Independent node implementation for DeepSeek models using rig-core

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataMap, NodeDefinition, NodeExecutable, NodeExecutionContext, NodeExecutionError,
  RegistrationError,
};
use rig::{
  client::CompletionClient,
  completion::Completion,
  message::{AssistantContent, Message},
};

use crate::constants::DEEPSEEK_MODEL_NODE_KIND;
use crate::llm::shared::{
  CommonLlmParameters, ModelCapabilities, UsageStats, create_base_node_definition, create_llm_execution_data_map,
  resolve_api_key, validate_api_key_resolved,
};
use crate::llm::{complation_error_2_execution_error, set_agent_builder};

/// DeepSeek-specific configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeepSeekNodeConfig {
  pub model: String,
  pub max_tokens: Option<u32>,
  pub temperature: Option<f64>,
  pub top_p: Option<u32>,
  pub stop_sequences: Option<Vec<String>>,
  pub common: CommonLlmParameters,
}

impl Default for DeepSeekNodeConfig {
  fn default() -> Self {
    Self {
      model: "deepseek-chat".to_string(),
      max_tokens: Some(128000),
      temperature: Some(0.7),
      top_p: None,
      stop_sequences: None,
      common: CommonLlmParameters::default(),
    }
  }
}

/// DeepSeek Node Implementation
#[derive(Debug)]
pub struct DeepseekV1 {
  pub definition: Arc<NodeDefinition>,
}

impl DeepseekV1 {
  pub fn new() -> Result<Self, RegistrationError> {
    let definition =
      create_base_node_definition(DEEPSEEK_MODEL_NODE_KIND, "DeepSeek 模型节点", "deepseek-chat", "Deepseek");
    Ok(Self { definition: Arc::new(definition) })
  }
}

impl TryFrom<NodeDefinition> for DeepseekV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(base) })
  }
}

#[async_trait]
impl NodeExecutable for DeepseekV1 {
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // Extract configuration
    let config: DeepSeekNodeConfig = context.get_parameters()?;

    // Resolve API key from various sources (direct value, environment variable, or credential reference)
    let resolved_api_key = resolve_api_key(&config.common.api_key, context).await?;

    // Validate resolved API key
    let api_key = validate_api_key_resolved(&resolved_api_key, "DeepseekNode")?;

    // Extract input messages
    let input_data = context.get_input_data(ConnectionKind::AiLM)?;

    // Create DeepSeek client using rig-core
    let deepseek_client = rig::providers::deepseek::Client::new(&api_key);

    // AgentBuilder
    let mut ab = deepseek_client.agent(&config.model);
    ab = set_agent_builder(&input_data, ab);

    let agent = ab.build();

    let prompt: Message = input_data
      .get_value("prompt")
      .map_err(|e| NodeExecutionError::InvalidInput(format!("Parameter prompt missing, error: {}", e)))?;
    let completion = agent.completion(prompt, vec![]).await.map_err(|e| NodeExecutionError::ExternalServiceError {
      service: format!("Deepseek agent completion error, error: {}", e),
    })?;

    let completion_response = completion
      .send()
      .await
      .map_err(|e| complation_error_2_execution_error(context.current_node_name().clone(), e))?;

    let usage_stats = UsageStats::from(completion_response.usage);

    // Extract response text from rig-core response
    // The completion response should be a string directly
    let choice = completion_response.choice.first();
    let response_text = match choice {
      AssistantContent::Text(text) => text.text,
      _ => "".to_string(),
    };

    // TODO Create model capabilities
    let capabilities = ModelCapabilities {
      chat: true,
      completion: true,
      tools: true,
      streaming: true,
      function_calling: true,
      vision: false,                  // DeepSeek is text-only
      max_context_length: Some(8192), // DeepSeek models have large context
      supported_formats: vec!["text".to_string(), "json".to_string()],
      json_mode: true,
      system_messages: true,
      temperature_control: true,
    };

    Ok(create_llm_execution_data_map(&response_text, &config.model, &self.definition.kind, usage_stats, capabilities))
  }

  fn definition(&self) -> Arc<NodeDefinition> {
    Arc::clone(&self.definition)
  }
}
