//! OpenAI LLM Node Implementation (gpt-4o-mini as default)
//!
//! 基于 rig-core 的 OpenAI 提供商，推广统一的 used_params/history_length 输出。

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::workflow::{
  ExecutionDataMap, FlowNode, NodeConnectionKind, NodeDescription, NodeExecutionContext, NodeExecutionError,
  RegistrationError,
};
use rig::{
  OneOrMany,
  client::CompletionClient,
  completion::Completion,
  message::{AssistantContent, Message as RigMessage, Text, UserContent},
};
use serde_json::json;

use crate::constants::OPENAI_MODEL_NODE_KIND;
use crate::lm::shared::{
  CommonLlmParameters, ModelCapabilities, UsageStats, create_base_node_definition, create_llm_execution_data_map,
  resolve_api_key, validate_api_key_resolved,
};
use crate::lm::{complation_error_2_execution_error, set_agent_builder};

/// OpenAI 节点配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenaiNodeConfig {
  pub model: String,
  pub max_tokens: Option<u32>,
  pub temperature: Option<f64>,
  pub top_p: Option<f64>,
  pub stop_sequences: Option<Vec<String>>,
  pub common: CommonLlmParameters,
}

impl Default for OpenaiNodeConfig {
  fn default() -> Self {
    Self {
      model: "gpt-4o-mini".to_string(),
      max_tokens: Some(4096),
      temperature: Some(0.7),
      top_p: Some(1.0),
      stop_sequences: None,
      common: CommonLlmParameters::default(),
    }
  }
}

/// OpenAI Node 实现
#[derive(Debug)]
pub struct OpenaiV1 {
  pub definition: Arc<NodeDescription>,
}

impl OpenaiV1 {
  /// 创建 OpenAI 节点定义
  pub fn new() -> Result<Self, RegistrationError> {
    let definition = create_base_node_definition(OPENAI_MODEL_NODE_KIND, "OpenAI 模型节点", "gpt-4o-mini", "OpenAI");
    Ok(Self { definition: Arc::new(definition) })
  }
}

impl TryFrom<NodeDescription> for OpenaiV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDescription) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(base) })
  }
}

#[async_trait]
impl FlowNode for OpenaiV1 {
  /// 执行 OpenAI LLM 节点
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // 解析参数
    let config: OpenaiNodeConfig = context.get_parameters()?;

    // 解析 API Key（支持 env/credential），并校验
    let resolved_api_key = resolve_api_key(&config.common.api_key, context).await?;
    let api_key = validate_api_key_resolved(&resolved_api_key, "OpenaiNode")?;

    // 读取输入
    let input_data = context.get_input_data(NodeConnectionKind::AiLanguageModel)?;

    // 创建 OpenAI Client 与 AgentBuilder
    let client = rig::providers::openai::Client::new(&api_key);
    let mut ab = client.agent(&config.model);
    ab = set_agent_builder(&input_data, ab);
    // 绑定参数：优先节点级配置，其次通用配置
    if let Some(t) = config.temperature.or(config.common.temperature) {
      ab = ab.temperature(t);
    }
    if let Some(mt) = config.max_tokens.or(config.common.max_tokens).map(|v| v as u64) {
      ab = ab.max_tokens(mt);
    }
    // 透传 top_p 与 stop（OpenAI 兼容字段名），采用 additional_params
    let mut extra = serde_json::Map::new();
    if let Some(tp) = config.top_p.or(config.common.top_p) {
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

    // 构造 prompt 与用户历史（最后一条 user 作为 prompt，其余 user 作为 chat_history）
    let input_json = input_data.json();
    let mut prompt_text = input_json.get("prompt").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();

    let mut chat_history: Vec<RigMessage> = Vec::new();
    if let Some(messages) = input_json.get("messages").and_then(|v| v.as_array()) {
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
      NodeExecutionError::ExternalServiceError { service: format!("OpenAI agent completion error, error: {}", e) }
    })?;

    let completion_response = completion
      .send()
      .await
      .map_err(|e| complation_error_2_execution_error(context.current_node_name().clone(), e))?;

    let usage_stats = UsageStats::from(completion_response.usage);

    // 文本回复提取
    let choice = completion_response.choice.first();
    let response_text = match choice {
      AssistantContent::Text(text) => text.text,
      _ => "".to_string(),
    };

    // 能力描述（示例值，可后续完善）
    let capabilities = ModelCapabilities {
      chat: true,
      completion: true,
      tools: true,
      streaming: true,
      function_calling: true,
      vision: false,
      max_context_length: Some(8192),
      supported_formats: vec!["text".to_string(), "json".to_string()],
      json_mode: true,
      system_messages: true,
      temperature_control: true,
    };

    // 观测字段：used_params 与 history_length
    let history_length = input_json.get("history_length").and_then(|v| v.as_u64());
    let used_params = json!({
      "temperature": config.temperature.or(config.common.temperature),
      "max_tokens": config.max_tokens.or(config.common.max_tokens),
      "top_p": config.top_p.or(config.common.top_p),
      "stop_sequences": config.stop_sequences,
    });

    Ok(create_llm_execution_data_map(
      &response_text,
      &config.model,
      &self.definition.node_type,
      usage_stats,
      capabilities,
      Some(used_params),
      history_length,
    ))
  }

  fn description(&self) -> Arc<NodeDescription> {
    Arc::clone(&self.definition)
  }
}
