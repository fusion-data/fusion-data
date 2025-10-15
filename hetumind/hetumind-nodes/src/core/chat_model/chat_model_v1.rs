use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{
    ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
    NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeGroupKind, NodeProperty, NodePropertyKind,
    OutputPortConfig, RegistrationError,
  },
};
use rig::providers::{anthropic::Client as AnthropicClient, openai::Client as OpenAIClient};
use serde_json::json;

use crate::constants::CHAT_MODEL_NODE_KIND;

use super::parameters::{
  AnthropicModel, LlmConfig, LocalModel, ModelCapabilities, ModelClient, OpenAIModel, StreamingResponse, UsageStats,
};

#[derive(Debug)]
pub struct ChatModelV1 {
  pub definition: Arc<NodeDefinition>,
}

impl ChatModelV1 {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = NodeDefinition::new(CHAT_MODEL_NODE_KIND, "LLM Chat Model");
    Self::try_from(base)
  }
}

impl TryFrom<NodeDefinition> for ChatModelV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .with_description("LLM 聊天模型节点，支持多种模型提供者")
      .add_group(NodeGroupKind::Transform)
      .with_icon("🧠")
      // 输入端口
      .add_input(InputPortConfig::builder()
          .kind(ConnectionKind::Main)
          .display_name("聊天消息输入")
          .required(true)
          .build())
      // 输出端口
      .add_output(OutputPortConfig::builder()
            .kind(ConnectionKind::Main)
            .display_name("模型响应")
            .build())
      // 模型实例端口
      .add_output(OutputPortConfig::builder()
            .kind(ConnectionKind::AiModel)
            .display_name("模型实例")
            .build())
      // 错误输出端口
      .add_output(OutputPortConfig::builder()
            .kind(ConnectionKind::Error)
            .display_name("错误输出")
            .build())
      // 参数
      .add_property(NodeProperty::builder()
            .name("provider")
            .kind(NodePropertyKind::String)
            .display_name("模型提供者")
            .value(json!("openai"))
            .required(true)
            .build())
      .add_property(NodeProperty::builder()
            .name("model")
            .kind(NodePropertyKind::String)
            .display_name("模型名称")
            .value(json!("gpt-3.5-turbo"))
            .required(true).build())
          .add_property(NodeProperty::builder()
            .name("credential_id")
            .kind(NodePropertyKind::String)
            .display_name("凭证ID")
            .description("用于获取API密钥的凭证ID，如果提供则优先使用凭证服务")
            .required(false).build())
          .add_property(NodeProperty::builder()
            .name("api_key")
            .kind(NodePropertyKind::String)
            .display_name("API 密钥")
            .description("API密钥，当未指定凭证ID时使用")
            .required(false).build())
          .add_property(NodeProperty::builder()
            .name("base_url")
            .kind(NodePropertyKind::String)
            .display_name("API 基础URL")
            .required(false).build())
          .add_property(NodeProperty::builder()
            .name("max_tokens")
            .kind(NodePropertyKind::Number)
            .display_name("最大令牌数")
            .value(json!(2000))
            .required(false).build())
          .add_property(NodeProperty::builder()
            .name("temperature")
            .kind(NodePropertyKind::Number)
            .display_name("温度参数")
            .value(json!(0.7))
            .required(false).build())
          .add_property(NodeProperty::builder()
            .name("stream")
            .kind(NodePropertyKind::Boolean)
            .display_name("是否启用流式响应")
            .value(json!(false))
            .required(false).build())
          .add_property(NodeProperty::builder()
            .name("timeout")
            .kind(NodePropertyKind::Number)
            .display_name("超时时间（秒）")
            .value(json!(60))
            .required(false)
            .build());
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl NodeExecutable for ChatModelV1 {
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // 1. 获取输入数据和配置
    let input_data = context.get_input_data("main")?;
    let config: LlmConfig = context.get_parameters()?;

    // 2. 创建模型客户端
    let model_client = self.create_model_client(&config).await?;

    // 3. 执行推理
    let response = if config.stream {
      self.execute_streaming_inference(&model_client, &input_data, &config).await?
    } else {
      self.execute_standard_inference(&model_client, &input_data, &config).await?
    };

    // 4. 构建输出数据
    Ok(make_execution_data_map(vec![
      ("main", ExecutionDataItems::Items(vec![ExecutionData::new_json(response.clone(), None)])),
      (
        "ai_model",
        ExecutionDataItems::Items(vec![ExecutionData::new_json(
          json!({
              "client": self.serialize_model_client(&model_client),
              "config": json!(config),
              "capabilities": self.get_model_capabilities(&config),
              "model_id": uuid::Uuid::new_v4().to_string(),
          }),
          None,
        )]),
      ),
    ]))
  }

  fn definition(&self) -> Arc<NodeDefinition> {
    Arc::clone(&self.definition)
  }
}

impl ChatModelV1 {
  async fn create_model_client(&self, config: &LlmConfig) -> Result<ModelClient, NodeExecutionError> {
    match config.provider.as_str() {
      "openai" => {
        let api_key = self.get_api_key_from_config_or_env(config, "OPENAI_API_KEY").await?;
        let client = OpenAIClient::new(&api_key);

        // 创建一个简化的ModelClient，不使用rig-core的model方法
        Ok(ModelClient::OpenAI(OpenAIModel { model: config.model.clone(), api_key, base_url: config.base_url.clone() }))
      }
      "anthropic" => {
        let api_key = self.get_api_key_from_config_or_env(config, "ANTHROPIC_API_KEY").await?;
        let client = AnthropicClient::new(&api_key);

        // 创建一个简化的ModelClient
        Ok(ModelClient::Anthropic(AnthropicModel {
          model: config.model.clone(),
          api_key,
          base_url: config.base_url.clone(),
        }))
      }
      "local" => {
        let endpoint = config
          .base_url
          .as_ref()
          .ok_or_else(|| NodeExecutionError::ConfigurationError("Local model endpoint not provided".to_string()))?;

        Ok(ModelClient::Local(LocalModel {
          endpoint: endpoint.clone(),
          model_name: config.model.clone(),
          api_key: config.api_key.clone(),
        }))
      }
      provider => Err(NodeExecutionError::ConfigurationError(format!("Unsupported provider: {}", provider))),
    }
  }

  /// 从配置或环境变量获取API密钥，支持凭证服务
  async fn get_api_key_from_config_or_env(
    &self,
    config: &LlmConfig,
    env_var: &str,
  ) -> Result<String, NodeExecutionError> {
    // 1. 优先使用配置中的API密钥
    if let Some(api_key) = &config.api_key {
      return Ok(api_key.clone());
    }

    // 2. 如果配置了凭证ID，尝试从凭证服务获取（这里需要注入凭证服务）
    if let Some(credential_id) = &config.credential_id {
      // TODO: 这里需要实际的凭证服务集成
      // 目前返回错误，提示需要实现凭证服务集成
      return Err(NodeExecutionError::ConfigurationError(format!(
        "Credential service integration not yet implemented for credential_id: {}",
        credential_id
      )));
    }

    // 3. 最后尝试从环境变量获取
    std::env::var(env_var).map_err(|_| {
      NodeExecutionError::ConfigurationError(format!(
        "API key not provided. Please set credential_id, api_key parameter, or {} environment variable",
        env_var
      ))
    })
  }

  async fn execute_standard_inference(
    &self,
    client: &ModelClient,
    input_data: &hetumind_core::workflow::ExecutionData,
    config: &LlmConfig,
  ) -> Result<JsonValue, NodeExecutionError> {
    let prompt = input_data
      .json()
      .get("prompt")
      .and_then(|v| v.as_str())
      .ok_or_else(|| NodeExecutionError::InvalidInput("No prompt provided".to_string()))?;

    // 模拟推理过程，实际实现需要集成具体的LLM客户端
    let mock_response = format!("这是来自 {} 模型的响应: {}", config.model, prompt);

    let usage = UsageStats {
      prompt_tokens: (prompt.len() as f32 / 4.0) as u32, // 估算token数
      completion_tokens: (mock_response.len() as f32 / 4.0) as u32,
      total_tokens: 0,
      estimated_cost: 0.0,
    };

    let total_tokens = usage.prompt_tokens + usage.completion_tokens;

    Ok(json!({
        "response": mock_response,
        "model": config.model,
        "provider": config.provider,
        "usage": usage,
        "timestamp": chrono::Utc::now().timestamp(),
        "request_id": uuid::Uuid::new_v4().to_string(),
    }))
  }

  async fn execute_streaming_inference(
    &self,
    client: &ModelClient,
    input_data: &hetumind_core::workflow::ExecutionData,
    config: &LlmConfig,
  ) -> Result<JsonValue, NodeExecutionError> {
    let prompt = input_data
      .json()
      .get("prompt")
      .and_then(|v| v.as_str())
      .ok_or_else(|| NodeExecutionError::InvalidInput("No prompt provided".to_string()))?;

    // 创建流式响应
    let stream_id = uuid::Uuid::new_v4().to_string();
    let mut streaming_response =
      StreamingResponse::new(stream_id.clone(), config.model.clone(), config.provider.clone());

    // 模拟流式响应生成过程
    let simulated_chunks = self.simulate_streaming_chunks(&config.model, prompt).await?;

    // 添加流式块
    for chunk in simulated_chunks {
      streaming_response.add_chunk(chunk);
    }

    // 完成流式响应
    streaming_response.finish();

    Ok(json!({
        "streaming_response": streaming_response,
        "model": config.model,
        "provider": config.provider,
        "streaming": true,
        "timestamp": chrono::Utc::now().timestamp(),
        "request_id": streaming_response.metadata.request_id,
    }))
  }

  /// 模拟流式响应块生成
  async fn simulate_streaming_chunks(&self, model: &str, prompt: &str) -> Result<Vec<String>, NodeExecutionError> {
    let full_response = format!("这是来自 {} 模型的流式响应: {}", model, prompt);

    // 将完整响应分割成多个块来模拟流式输出
    let chunk_size = 10; // 每个块10个字符
    let mut chunks = Vec::new();

    for (i, chunk) in full_response.chars().collect::<Vec<_>>().chunks(chunk_size).enumerate() {
      let chunk_str: String = chunk.iter().collect();

      // 模拟网络延迟
      tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

      chunks.push(chunk_str);

      // 最后一个块添加完成标记
      if i == (full_response.chars().count() + chunk_size - 1) / chunk_size - 1 {
        chunks.push("[DONE]".to_string());
      }
    }

    Ok(chunks)
  }

  fn serialize_model_client(&self, client: &ModelClient) -> JsonValue {
    match client {
      ModelClient::OpenAI(model) => json!({
          "type": "openai",
          "model": model.model,
          "base_url": model.base_url,
      }),
      ModelClient::Anthropic(model) => json!({
          "type": "anthropic",
          "model": model.model,
          "base_url": model.base_url,
      }),
      ModelClient::Local(model) => json!({
          "type": "local",
          "endpoint": model.endpoint,
          "model_name": model.model_name,
      }),
      ModelClient::Custom(model) => json!({
          "type": "custom",
          "provider": model.provider,
          "config": model.config,
      }),
    }
  }

  fn get_model_capabilities(&self, config: &LlmConfig) -> ModelCapabilities {
    match config.provider.as_str() {
      "openai" => ModelCapabilities {
        chat: true,
        completion: true,
        tools: true,
        streaming: true,
        function_calling: true,
        vision: config.model.contains("vision") || config.model.contains("4o"),
        max_context_length: Some(self.get_context_length(&config.model)),
      },
      "anthropic" => ModelCapabilities {
        chat: true,
        completion: false,
        tools: true,
        streaming: true,
        function_calling: true,
        vision: config.model.contains("vision"),
        max_context_length: Some(self.get_context_length(&config.model)),
      },
      "local" => ModelCapabilities {
        chat: true,
        completion: false,
        tools: false,     // 取决于具体实现
        streaming: false, // 取决于具体实现
        function_calling: false,
        vision: false,
        max_context_length: None, // 取决于具体模型
      },
      _ => ModelCapabilities::default(),
    }
  }

  fn get_context_length(&self, model: &str) -> usize {
    // 简化的上下文长度映射
    if model.contains("gpt-4") {
      8192
    } else if model.contains("gpt-3.5") {
      4096
    } else if model.contains("claude-3") {
      200000
    } else {
      4096 // 默认值
    }
  }
}

// Helper function for creating ExecutionDataMap
fn make_execution_data_map(data: Vec<(&str, ExecutionDataItems)>) -> ExecutionDataMap {
  use fusion_common::ahash::{HashMap, HashMapExt};
  let mut map = HashMap::new();
  for (key, value) in data {
    let connection_kind = match key {
      "main" => ConnectionKind::Main,
      "error" => ConnectionKind::Error,
      "ai_agent" => ConnectionKind::AiAgent,
      "ai_tool" => ConnectionKind::AiTool,
      "ai_model" => ConnectionKind::AiModel,
      "ai_output_parser" => ConnectionKind::AiOutputParser,
      "ai_memory" => ConnectionKind::AiMemory,
      "ai_document" => ConnectionKind::AiDocument,
      "ai_embedding" => ConnectionKind::AiEmbedding,
      "ai_retriever" => ConnectionKind::AiRetriever,
      "ai_vector_store" => ConnectionKind::AiVectorStore,
      "ai_reranker" => ConnectionKind::AiReranker,
      "ai_text_splitter" => ConnectionKind::AiTextSplitter,
      _ => continue, // Skip unknown connection types
    };
    map.insert(connection_kind, vec![value]);
  }
  map
}
