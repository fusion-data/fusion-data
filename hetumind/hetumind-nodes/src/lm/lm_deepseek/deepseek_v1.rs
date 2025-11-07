use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  LLMConfig, LLMResponse, LLMSubNodeProvider, Message, NodeConnectionKind, NodeDescription, NodeExecutionError,
  NodeGroupKind, OutputPortConfig, SubNode, SubNodeType,
};
use rig::{
  OneOrMany,
  client::CompletionClient,
  completion::Completion,
  message::{AssistantContent, Message as RigMessage, Text, UserContent},
};
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
  /// 调用 LLM（占位实现）：返回固定响应，后续接入 rig-core Agent/Client
  async fn call_llm(&self, messages: Vec<Message>, config: LLMConfig) -> Result<LLMResponse, NodeExecutionError> {
    // 解析 API Key：优先使用配置中的 api_key；支持 ${env:VAR} 引用；否则回退到环境变量 DEEPSEEK_API_KEY
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

    // 创建 DeepSeek 客户端与 Agent
    let client = rig::providers::deepseek::Client::new(&api_key);
    let model_name = config.model.clone();
    let mut ab = client.agent(&model_name);
    // 绑定参数：temperature / max_tokens
    if let Some(t) = config.temperature {
      ab = ab.temperature(t);
    }
    if let Some(mt) = config.max_tokens.map(|v| v as u64) {
      ab = ab.max_tokens(mt);
    }
    // 透传 top_p 与 stop（OpenAI 兼容字段名），采用 additional_params
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

    // 构造 prompt 与 chat_history：最后一条 user 作为 prompt，其余 user/assistant 作为历史
    let last_user_index = messages.iter().rposition(|m| m.role == "user");
    let mut chat_history: Vec<RigMessage> = Vec::new();
    let mut prompt_text = String::new();
    for (idx, m) in messages.iter().enumerate() {
      match m.role.as_str() {
        "user" => {
          if Some(idx) == last_user_index {
            prompt_text = m.content.clone();
          } else {
            chat_history
              .push(RigMessage::User { content: OneOrMany::one(UserContent::Text(Text { text: m.content.clone() })) });
          }
        }
        "assistant" => {
          chat_history.push(RigMessage::Assistant {
            id: Some(format!("assistant_{}", idx)),
            content: OneOrMany::one(AssistantContent::Text(Text { text: m.content.clone() })),
          });
        }
        _ => {}
      }
    }
    let prompt = RigMessage::User { content: OneOrMany::one(UserContent::Text(Text { text: prompt_text })) };

    // 执行补全并发送
    let completion = agent.completion(prompt, chat_history).await.map_err(|e| {
      NodeExecutionError::ExternalServiceError { service: format!("Deepseek agent completion error: {}", e) }
    })?;
    let resp = completion.send().await.map_err(|e| NodeExecutionError::ExternalServiceError {
      service: format!("Deepseek completion send error: {}", e),
    })?;

    // 提取文本回复
    let choice = resp.choice.first();
    let response_text = match choice {
      AssistantContent::Text(text) => text.text,
      _ => String::new(),
    };

    // 提取使用统计
    let usage = resp.usage;
    let usage_stats = hetumind_core::workflow::UsageStats {
      prompt_tokens: usage.input_tokens as u32,
      completion_tokens: usage.output_tokens as u32,
      total_tokens: usage.total_tokens as u32,
    };

    Ok(LLMResponse { content: response_text, role: "assistant".to_string(), usage: Some(usage_stats) })
  }
}
