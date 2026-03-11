//! Moonshot LLM Node Implementation (kimi-k2-0905-preview)
//!
//! 基于 rig-core 的 Moonshot 提供商，推广统一的 used_params/history_length 输出。

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::workflow::{
  ExecutionDataMap, FlowNode, NodeConnectionKind, NodeDescription, NodeExecutionContext, NodeExecutionError,
  RegistrationError,
};
use rig::client::CompletionClient;
use rig::completion::{Chat, Completion};
use serde_json::json;

use crate::constants::MOONSHOT_MODEL_NODE_KIND;
use crate::lm::set_agent_builder;
use crate::lm::shared::{
  CommonLlmParameters, ModelCapabilities, UsageStats, create_base_node_definition, create_llm_execution_data_map,
  resolve_api_key, validate_api_key_resolved,
};

/// Moonshot 节点配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MoonshotNodeConfig {
  pub model: String,
  pub max_tokens: Option<u32>,
  pub temperature: Option<f64>,
  pub top_p: Option<f64>,
  pub stop_sequences: Option<Vec<String>>,
  pub common: CommonLlmParameters,
}

impl Default for MoonshotNodeConfig {
  fn default() -> Self {
    Self {
      model: "kimi-k2-0905-preview".to_string(),
      max_tokens: Some(4096),
      temperature: Some(0.7),
      top_p: Some(1.0),
      stop_sequences: None,
      common: CommonLlmParameters::default(),
    }
  }
}

/// Moonshot Node 实现
#[derive(Debug)]
pub struct MoonshotV1 {
  pub definition: Arc<NodeDescription>,
}

impl MoonshotV1 {
  /// 创建 Moonshot 节点定义
  pub fn new() -> Result<Self, RegistrationError> {
    let definition =
      create_base_node_definition(MOONSHOT_MODEL_NODE_KIND, "Moonshot 模型节点", "kimi-k2-0905-preview", "Moonshot");
    Ok(Self { definition: Arc::new(definition) })
  }
}

impl TryFrom<NodeDescription> for MoonshotV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDescription) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(base) })
  }
}

#[async_trait]
impl FlowNode for MoonshotV1 {
  /// 执行 Moonshot LLM 节点
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // 解析参数
    let config: MoonshotNodeConfig = context.get_parameters()?;

    // 解析 API Key（支持 env/credential），并校验
    let resolved_api_key = resolve_api_key(&config.common.api_key, context).await?;
    let api_key = validate_api_key_resolved(&resolved_api_key, "MoonshotNode")?;

    // 读取输入
    let input_data = context.get_input_data(NodeConnectionKind::AiLanguageModel)?;

    // 创建 Moonshot Client
    let client = rig::providers::moonshot::Client::new(&api_key)
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("Failed to create Moonshot client: {}", e)))?;
    let model: rig::providers::moonshot::CompletionModel<reqwest::Client> = client.completion_model(&config.model);

    // 创建 Agent
    let mut ab = rig::agent::AgentBuilder::new(model);
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

    // 构造 prompt 与用户历史
    let input_json = input_data.json();
    let messages = extract_chat_messages(input_json)?;

    // 使用 chat 方法直接获取响应
    let response_text = agent
      .chat(messages.prompt, messages.history)
      .await
      .map_err(|e| NodeExecutionError::ExternalServiceError { service: format!("Moonshot agent chat error: {}", e) })?;

    // 获取使用统计
    let usage_stats = extract_usage_from_agent(&agent).await.unwrap_or(UsageStats {
      prompt_tokens: 0,
      completion_tokens: 0,
      total_tokens: 0,
      estimated_cost: 0.0,
    });

    // 能力描述
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

    // 观测字段
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

/// 提取聊天消息：最后一个 user 作为 prompt，其他消息作为 history
struct ChatMessages {
  prompt: rig::message::Message,
  history: Vec<rig::message::Message>,
}

fn extract_chat_messages(input_json: &serde_json::Value) -> Result<ChatMessages, NodeExecutionError> {
  let mut prompt_text = input_json.get("prompt").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();

  let mut history: Vec<rig::message::Message> = Vec::new();
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
            history.push(rig::message::Message::user(content.to_string()));
          }
        }
        "assistant" => {
          history.push(rig::message::Message::assistant(content.to_string()));
        }
        _ => {}
      }
    }
  }

  let prompt = rig::message::Message::user(prompt_text);
  Ok(ChatMessages { prompt, history })
}

/// 从 Agent 获取使用统计
async fn extract_usage_from_agent<M>(agent: &rig::agent::Agent<M>) -> Option<UsageStats>
where
  M: rig::completion::CompletionModel,
{
  let prompt = rig::message::Message::user("");
  if let Ok(request_builder) = agent.completion(prompt, vec![]).await
    && let Ok(response) = request_builder.send().await
  {
    let usage = response.usage;
    return Some(UsageStats {
      prompt_tokens: usage.input_tokens,
      completion_tokens: usage.output_tokens,
      total_tokens: usage.total_tokens,
      estimated_cost: 0.0,
    });
  }
  None
}
