//! DeepSeek LLM Node Implementation
//!
//! Independent node implementation for DeepSeek models using rig-core

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataMap, FlowNode, NodeDefinition, NodeExecutionContext, NodeExecutionError,
  RegistrationError,
};
use rig::{
  OneOrMany,
  client::CompletionClient,
  completion::Completion,
  message::{AssistantContent, Message as RigMessage, Text, UserContent},
};

use crate::constants::DEEPSEEK_MODEL_NODE_KIND;
use crate::llm::shared::{
  CommonLlmParameters, ModelCapabilities, UsageStats, create_base_node_definition, create_llm_execution_data_map,
  resolve_api_key, validate_api_key_resolved,
};
use crate::llm::{complation_error_2_execution_error, set_agent_builder};
use serde_json::json;

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
impl FlowNode for DeepseekV1 {
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
    // 绑定参数：优先使用节点级配置，其次使用通用配置
    if let Some(t) = config.temperature.or(config.common.temperature) {
      ab = ab.temperature(t);
    }
    if let Some(mt) = config.max_tokens.or(config.common.max_tokens).map(|v| v as u64) {
      ab = ab.max_tokens(mt);
    }
    // 透传 top_p 与 stop（OpenAI 兼容字段名），采用 additional_params
    let mut extra = serde_json::Map::new();
    if let Some(tp) = config.top_p.map(|v| v as f64).or(config.common.top_p) {
      extra.insert("top_p".to_string(), json!(tp));
    }
    if let Some(stops) = config.stop_sequences.as_ref()
      && !stops.is_empty()
    {
      extra.insert("stop".to_string(), json!(stops));
    }
    if !extra.is_empty() {
      ab = ab.additional_params(json!(extra));
    }

    let agent = ab.build();

    // 构造 prompt 与 chat_history：优先从 messages 中提取；否则回退到 prompt 字段
    let input_json = input_data.json();
    let mut prompt_text = input_json.get("prompt").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();

    let mut chat_history: Vec<RigMessage> = Vec::new();
    if let Some(messages) = input_json.get("messages").and_then(|v| v.as_array()) {
      // 找到最后一条 user 消息作为 prompt
      let last_user_index = messages.iter().rposition(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"));

      for (idx, m) in messages.iter().enumerate() {
        let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
        match role {
          "user" => {
            if Some(idx) == last_user_index {
              prompt_text = content.to_string();
            } else {
              chat_history.push(RigMessage::User {
                content: OneOrMany::one(UserContent::Text(Text { text: content.to_string() })),
              });
            }
          }
          "assistant" => {
            chat_history.push(RigMessage::Assistant {
              id: Some(format!("assistant_{}", idx)),
              content: OneOrMany::one(AssistantContent::Text(Text { text: content.to_string() })),
            });
          }
          _ => {}
        }
      }
    }

    let prompt: RigMessage =
      RigMessage::User { content: OneOrMany::one(UserContent::Text(Text { text: prompt_text })) };
    let completion = agent.completion(prompt, chat_history).await.map_err(|e| {
      NodeExecutionError::ExternalServiceError { service: format!("Deepseek agent completion error, error: {}", e) }
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

    // 组装 used_params 与 history_length（用于观测与复现）
    let input_json = input_data.json();
    let history_length = input_json.get("history_length").and_then(|v| v.as_u64());

    // 参数优先级：节点级 > 通用参数 > 输入 JSON > 默认值
    let used_params = json!({
      "temperature": config.temperature.or(config.common.temperature),
      "max_tokens": config.max_tokens.or(config.common.max_tokens),
      // 修正 top_p 的优先级为节点级优先
      "top_p": config.top_p.map(|v| v as f64).or(config.common.top_p),
      "stop_sequences": config.stop_sequences,
    });

    Ok(create_llm_execution_data_map(
      &response_text,
      &config.model,
      &self.definition.kind,
      usage_stats,
      capabilities,
      Some(used_params),
      history_length,
    ))
  }

  fn definition(&self) -> Arc<NodeDefinition> {
    Arc::clone(&self.definition)
  }
}
