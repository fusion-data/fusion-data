//! Shared functionality for LLM nodes
//!
//! Common parameters, error handling, and utility functions shared across all LLM providers.

use ahash::HashMap;
use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{
    ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
    NodeExecutionContext, NodeExecutionError, NodeGroupKind, NodeKind, NodeName, NodeProperty, NodePropertyKind,
    OutputPortConfig,
  },
};
use rig::{
  agent::AgentBuilder,
  completion::{CompletionError, CompletionModel},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Common LLM node parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommonLlmParameters {
  /// API key (optional, can use environment variables or credential references)
  pub api_key: Option<String>,
  /// Maximum tokens to generate
  pub max_tokens: Option<u32>,
  /// Temperature for response randomness
  pub temperature: Option<f64>,
  /// Top-p sampling parameter
  pub top_p: Option<f64>,
  /// Enable streaming responses
  pub stream: Option<bool>,
  /// Timeout in seconds
  pub timeout: Option<u32>,
  /// Additional model-specific parameters
  pub extra_params: Option<JsonValue>,
}

impl Default for CommonLlmParameters {
  fn default() -> Self {
    Self {
      api_key: None,
      max_tokens: Some(4096),
      temperature: Some(0.7),
      top_p: Some(1.0),
      stream: Some(false),
      timeout: Some(60),
      extra_params: None,
    }
  }
}

/// Create a base node definition with common configuration
pub fn create_base_node_definition(
  node_kind: &str,
  description: &str,
  model_name: &str,
  model_provider: &str,
) -> NodeDefinition {
  NodeDefinition::new(node_kind, description)
    .with_version(Version::new(1, 0, 0))
    .with_description(format!("{} ({})", description, model_provider))
    .add_group(NodeGroupKind::Transform)
    .with_icon("🧠")
    // Input port for messages
    .add_input(InputPortConfig::new(ConnectionKind::Main, "聊天消息输入")
        .with_required(true))
    // Output ports
    .add_output(OutputPortConfig::new(ConnectionKind::Main, "模型响应"))
    .add_output(OutputPortConfig::new(ConnectionKind::AiLM, "模型实例"))
    .add_output(OutputPortConfig::new(ConnectionKind::Error, "错误输出"))
    // Common model configuration
    .add_property(NodeProperty::new(NodePropertyKind::String)
        .with_name("model")
        .with_display_name("模型名称")
        .with_value(json!(model_name))
        .with_required(true))
    .add_property(NodeProperty::new(NodePropertyKind::String)
        .with_name("api_key")
        .with_display_name("API 密钥")
        .with_description("API密钥，留空则使用环境变量或凭证服务")
        .with_required(false))
    .add_property(NodeProperty::new(NodePropertyKind::Number)
        .with_name("max_tokens")
        .with_display_name("最大令牌数")
        .with_value(json!(4096))
        .with_required(false))
    .add_property(NodeProperty::new(NodePropertyKind::Number)
        .with_name("temperature")
        .with_display_name("温度参数")
        .with_value(json!(0.7))
        .with_required(false))
    .add_property(NodeProperty::new(NodePropertyKind::Number)
        .with_name("top_p")
        .with_display_name("Top-p")
        .with_value(json!(1.0))
        .with_required(false))
    .add_property(NodeProperty::new(NodePropertyKind::Boolean)
        .with_name("stream")
        .with_display_name("启用流式响应")
        .with_value(json!(false))
        .with_required(false))
    .add_property(NodeProperty::new(NodePropertyKind::Number)
        .with_name("timeout")
        .with_display_name("超时时间（秒）")
        .with_value(json!(60))
        .with_required(false))
}

/// Set agent builder with input data
pub fn set_agent_builder<M>(input_data: &ExecutionData, mut ab: AgentBuilder<M>) -> AgentBuilder<M>
where
  M: CompletionModel,
{
  let json = input_data.json();
  if let Some(system_prompt) = json.get("system_prompt").and_then(|v| v.as_str()) {
    ab = ab.preamble(system_prompt);
  }
  // 绑定可选参数：temperature 与 max_tokens（若存在）
  if let Some(temperature) = json.get("temperature").and_then(|v| v.as_f64()) {
    ab = ab.temperature(temperature);
  }
  if let Some(max_tokens) = json.get("max_tokens").and_then(|v| v.as_u64()) {
    ab = ab.max_tokens(max_tokens);
  }
  // 透传 provider-specific 参数：top_p 与 stop（OpenAI 兼容字段）
  let mut extra = serde_json::Map::new();
  if let Some(tp) = json.get("top_p").and_then(|v| v.as_f64()) {
    extra.insert("top_p".to_string(), json!(tp));
  }
  if let Some(stops) = json.get("stop_sequences").and_then(|v| v.as_array()) {
    let arr: Vec<String> = stops.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
    if !arr.is_empty() {
      extra.insert("stop".to_string(), json!(arr));
    }
  }
  if !extra.is_empty() {
    ab = ab.additional_params(json!(extra));
  }
  ab
}

/// Extract messages from input data with multiple format support
pub fn extract_messages_from_input(input_data: &ExecutionData) -> Result<Vec<serde_json::Value>, NodeExecutionError> {
  let input_json = input_data.json();

  // Try different message formats
  if let Some(messages) = input_json.get("messages").and_then(|v| v.as_array()) {
    return Ok(messages.clone());
  }

  if let Some(messages) = input_json.get("data").and_then(|v| v.as_array()) {
    return Ok(messages.clone());
  }

  // Single message case
  if let Some(content) = input_json.get("prompt").and_then(|v| v.as_str()) {
    return Ok(vec![json!({
      "role": "user",
      "content": content
    })]);
  }

  if let Some(content) = input_json.get("content").and_then(|v| v.as_str()) {
    return Ok(vec![json!({
      "role": "user",
      "content": content
    })]);
  }

  if let Some(text) = input_json.get("text").and_then(|v| v.as_str()) {
    return Ok(vec![json!({
      "role": "user",
      "content": text
    })]);
  }

  Err(NodeExecutionError::InvalidInput(
    "Unable to extract messages from input data. Expected 'messages', 'data', 'prompt', 'content', or 'text' field."
      .to_string(),
  ))
}

/// Usage statistics for LLM responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
  pub prompt_tokens: u64,
  pub completion_tokens: u64,
  pub total_tokens: u64,
  pub estimated_cost: f64,
}

impl From<rig::completion::Usage> for UsageStats {
  fn from(value: rig::completion::Usage) -> Self {
    Self {
      prompt_tokens: value.input_tokens,
      completion_tokens: value.output_tokens,
      total_tokens: value.total_tokens,
      estimated_cost: 0.0,
    }
  }
}

/// Model capabilities for LLM responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
  pub chat: bool,
  pub completion: bool,
  pub tools: bool,
  pub streaming: bool,
  pub function_calling: bool,
  pub vision: bool,
  pub max_context_length: Option<u64>,
  pub supported_formats: Vec<String>,
  pub json_mode: bool,
  pub system_messages: bool,
  pub temperature_control: bool,
}

/// Create standard execution data map for LLM responses
pub fn create_llm_execution_data_map(
  response_content: &str,
  model_name: &str,
  node_kind: &NodeKind,
  usage_stats: UsageStats,
  capabilities: ModelCapabilities,
  used_params: Option<serde_json::Value>,
  history_length: Option<u64>,
) -> ExecutionDataMap {
  let mut map = HashMap::default();

  // Main response
  let mut model_info = json!({
    "content": response_content,
    "model": model_name,
    "node_kind": node_kind,
    "usage": usage_stats,
    "streaming": false,
    "capabilities": capabilities
  });

  if let Some(params) = used_params {
    model_info["used_params"] = params;
  }
  if let Some(hlen) = history_length {
    model_info["history_length"] = json!(hlen);
  }

  map.insert(ConnectionKind::AiLM, vec![ExecutionDataItems::Items(vec![ExecutionData::new_json(model_info, None)])]);

  map
}

/// Convert NodeExecutionError to JSON for error output
pub fn create_error_execution_data(error: &NodeExecutionError) -> ExecutionDataMap {
  use fusion_common::ahash::{HashMap, HashMapExt};
  let mut map = HashMap::new();

  let error_json = json!({
    "error": error.to_string(),
    "error_type": "execution_error",
    "timestamp": chrono::Utc::now().timestamp()
  });

  map.insert(ConnectionKind::Error, vec![ExecutionDataItems::Items(vec![ExecutionData::new_json(error_json, None)])]);

  map
}

/// Resolve API key from various sources (direct value, environment variable, or credential reference)
pub async fn resolve_api_key(
  api_key_config: &Option<String>,
  context: &NodeExecutionContext,
) -> Result<Option<String>, NodeExecutionError> {
  match api_key_config {
    None => Ok(None),
    Some(key) => {
      if key.starts_with("${env:") && key.ends_with('}') {
        // Environment variable reference: ${env:VARIABLE_NAME}
        let env_var = &key[6..key.len() - 1];
        std::env::var(env_var)
          .map(Some)
          .map_err(|_| NodeExecutionError::ConfigurationError(format!("Environment variable '{}' not found", env_var)))
      } else if key.starts_with("${CREDENTIAL:") && key.ends_with('}') {
        // Credential reference: ${CREDENTIAL:credential_name}
        let credential_name = &key[13..key.len() - 1];
        resolve_credential_api_key(credential_name, context).await
      } else {
        // Direct API key value
        Ok(Some(key.clone()))
      }
    }
  }
}

/// Resolve API key from credential service
async fn resolve_credential_api_key(
  credential_name: &str,
  _context: &NodeExecutionContext,
) -> Result<Option<String>, NodeExecutionError> {
  // For now, we'll simulate credential resolution
  // In a real implementation, this would:
  // 1. Query the credential service for the credential by name
  // 2. Decrypt the credential data
  // 3. Extract the API key

  // Mock credential resolution for testing
  match credential_name {
    "deepseek_api_key" => Ok(Some("sk-mock-deepseek-api-key-from-credential".to_string())),
    _ => Err(NodeExecutionError::ConfigurationError(format!("Credential '{}' not found", credential_name))),
  }
}

/// Enhanced API key validation with credential support
pub fn validate_api_key_resolved(api_key: &Option<String>, node_name: &str) -> Result<String, NodeExecutionError> {
  match api_key {
    None => Err(NodeExecutionError::ExecutionFailed {
      node_name: node_name.into(),
      message: Some(
        "API key is required. Please provide an API key or configure environment variable/credential.".to_string(),
      ),
    }),
    Some(key) if key.trim().is_empty() => Err(NodeExecutionError::ExecutionFailed {
      node_name: node_name.into(),
      message: Some("API key cannot be empty".to_string()),
    }),
    Some(key) => Ok(key.clone()),
  }
}

pub fn complation_error_2_execution_error(node_name: NodeName, error: CompletionError) -> NodeExecutionError {
  NodeExecutionError::ExecutionFailed { node_name, message: Some(error.to_string()) }
}
