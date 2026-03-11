use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  LLMConfig, LLMResponse, LLMSubNodeProvider, Message, NodeConnectionKind, NodeDescription, NodeExecutionError,
  NodeGroupKind, OutputPortConfig, SubNode, SubNodeType,
};
use rig::client::CompletionClient;
use rig::completion::{Chat, Completion};
use serde_json::json;

use crate::constants::DEEPSEEK_MODEL_NODE_KIND;

/// DeepSeek 模型 Supplier（LLMSubNodeProvider）
pub struct DeepseekModelV1 {
  definition: Arc<NodeDescription>,
}

impl Default for DeepseekModelV1 {
  fn default() -> Self {
    Self::new()
  }
}

impl DeepseekModelV1 {
  pub fn new() -> Self {
    Self { definition: Arc::new(Self::create_definition()) }
  }

  fn create_definition() -> NodeDescription {
    NodeDescription::new(DEEPSEEK_MODEL_NODE_KIND, "DeepSeek Model")
      .with_version(Version::new(1, 0, 0))
      .add_group(NodeGroupKind::Transform)
      .with_description("DeepSeek LLM provider for Agent consumption")
      .add_output(OutputPortConfig::new(NodeConnectionKind::AiLanguageModel, "Model"))
  }
}

#[async_trait]
impl SubNode for DeepseekModelV1 {
  fn provider_type(&self) -> SubNodeType {
    SubNodeType::LLM
  }
  fn description(&self) -> Arc<NodeDescription> {
    self.definition.clone()
  }

  async fn initialize(&self) -> Result<(), NodeExecutionError> {
    Ok(())
  }

  /// 返回 Any 引用用于安全 downcast（typed 获取）
  fn as_any(&self) -> &dyn std::any::Any {
    self
  }
}

#[async_trait]
impl LLMSubNodeProvider for DeepseekModelV1 {
  /// 调用 LLM：使用 rig 0.27 API
  async fn call_llm(&self, messages: Vec<Message>, config: LLMConfig) -> Result<LLMResponse, NodeExecutionError> {
    // 解析 API Key
    let api_key = match config.api_key.as_ref() {
      Some(k) if k.starts_with("${env:") && k.ends_with('}') => {
        let env_var = &k[6..k.len() - 1];
        std::env::var(env_var).map_err(|_| {
          NodeExecutionError::ConfigurationError(format!("Environment variable '{}' not found", env_var))
        })?
      }
      Some(k) if !k.is_empty() => k.clone(),
      _ => std::env::var("DEEPSEEK_API_KEY").map_err(|_| {
        NodeExecutionError::ConfigurationError("Missing DEEPSEEK_API_KEY environment variable".to_string())
      })?,
    };

    // 创建 DeepSeek 客户端
    let client = rig::providers::deepseek::Client::new(&api_key)
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("Failed to create DeepSeek client: {}", e)))?;
    let model: rig::providers::deepseek::CompletionModel<reqwest::Client> = client.completion_model(&config.model);

    // 创建 Agent
    let mut ab = rig::agent::AgentBuilder::new(model);

    // 绑定参数
    if let Some(t) = config.temperature {
      ab = ab.temperature(t);
    }
    if let Some(mt) = config.max_tokens.map(|v| v as u64) {
      ab = ab.max_tokens(mt);
    }
    // 透传 top_p 与 stop
    let mut extra = serde_json::Map::new();
    if let Some(tp) = config.top_p.map(|v| v as f64) {
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
    // 将 role=system 的消息作为 preamble 注入
    let system_prompts: Vec<&str> = messages
      .iter()
      .filter(|m| m.role == "system")
      .map(|m| m.content.as_str())
      .filter(|s| !s.is_empty())
      .collect();
    if !system_prompts.is_empty() {
      let preamble = system_prompts.join("\n\n");
      ab = ab.preamble(&preamble);
    }
    let agent = ab.build();

    // 构造 prompt 与 chat_history
    let (prompt, chat_history) = convert_messages_to_rig_format(messages)?;

    // 使用 chat 方法直接获取响应
    let response_text = agent
      .chat(prompt, chat_history)
      .await
      .map_err(|e| NodeExecutionError::ExternalServiceError { service: format!("DeepSeek agent chat error: {}", e) })?;

    // 获取使用统计
    let usage_stats = extract_usage_from_agent(&agent)
      .await
      .map(|u| hetumind_core::workflow::UsageStats {
        prompt_tokens: u.prompt_tokens as u32,
        completion_tokens: u.completion_tokens as u32,
        total_tokens: u.total_tokens as u32,
      })
      .unwrap_or_else(|| hetumind_core::workflow::UsageStats {
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
      });

    Ok(LLMResponse { content: response_text, role: "assistant".to_string(), usage: Some(usage_stats) })
  }
}

/// 转换消息到 rig 格式
fn convert_messages_to_rig_format(
  messages: Vec<Message>,
) -> Result<(rig::message::Message, Vec<rig::message::Message>), NodeExecutionError> {
  let last_user_index = messages.iter().rposition(|m| m.role == "user");
  let mut prompt_text = String::new();
  let mut history: Vec<rig::message::Message> = Vec::new();

  for (idx, m) in messages.iter().enumerate() {
    match m.role.as_str() {
      "user" => {
        if Some(idx) == last_user_index {
          prompt_text = m.content.clone();
        } else {
          history.push(rig::message::Message::user(m.content.clone()));
        }
      }
      "assistant" => {
        history.push(rig::message::Message::assistant(m.content.clone()));
      }
      _ => {}
    }
  }

  let prompt = rig::message::Message::user(prompt_text);
  Ok((prompt, history))
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

struct UsageStats {
  prompt_tokens: u64,
  completion_tokens: u64,
  total_tokens: u64,
  #[allow(dead_code)]
  estimated_cost: f64,
}
