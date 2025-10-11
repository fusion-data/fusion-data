use hetumind_core::types::JsonValue;
use serde::{Deserialize, Serialize};

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
  /// 模型提供者
  pub provider: String,
  /// 模型名称
  pub model: String,
  /// 凭证ID（用于从凭证服务获取API密钥）
  pub credential_id: Option<String>,
  /// API 密钥
  pub api_key: Option<String>,
  /// API 基础URL
  pub base_url: Option<String>,
  /// 最大令牌数
  pub max_tokens: u32,
  /// 温度参数
  pub temperature: f64,
  /// 是否启用流式响应
  pub stream: bool,
  /// 超时时间（秒）
  pub timeout: Option<u32>,
  /// 额外的模型参数
  pub extra_params: Option<JsonValue>,
}

impl Default for LlmConfig {
  fn default() -> Self {
    Self {
      provider: "openai".to_string(),
      model: "gpt-3.5-turbo".to_string(),
      credential_id: None,
      api_key: None,
      base_url: None,
      max_tokens: 2000,
      temperature: 0.7,
      stream: false,
      timeout: Some(60),
      extra_params: None,
    }
  }
}

/// 模型客户端枚举
#[derive(Debug, Clone)]
pub enum ModelClient {
  OpenAI(OpenAIModel),
  Anthropic(AnthropicModel),
  Local(LocalModel),
  Custom(CustomModel),
}

/// OpenAI 模型
#[derive(Debug, Clone)]
pub struct OpenAIModel {
  pub model: String,
  pub api_key: String,
  pub base_url: Option<String>,
}

/// Anthropic 模型
#[derive(Debug, Clone)]
pub struct AnthropicModel {
  pub model: String,
  pub api_key: String,
  pub base_url: Option<String>,
}

/// 本地模型
#[derive(Debug, Clone)]
pub struct LocalModel {
  pub endpoint: String,
  pub model_name: String,
  pub api_key: Option<String>,
}

/// 自定义模型
#[derive(Debug, Clone)]
pub struct CustomModel {
  pub provider: String,
  pub config: JsonValue,
}

/// 模型能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
  /// 支持聊天
  pub chat: bool,
  /// 支持补全
  pub completion: bool,
  /// 支持工具调用
  pub tools: bool,
  /// 支持流式响应
  pub streaming: bool,
  /// 支持函数调用
  pub function_calling: bool,
  /// 支持视觉输入
  pub vision: bool,
  /// 最大上下文长度
  pub max_context_length: Option<usize>,
}

impl Default for ModelCapabilities {
  fn default() -> Self {
    Self {
      chat: true,
      completion: false,
      tools: true,
      streaming: true,
      function_calling: true,
      vision: false,
      max_context_length: Some(4096),
    }
  }
}

/// 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
  /// 提示令牌数
  pub prompt_tokens: u32,
  /// 完成令牌数
  pub completion_tokens: u32,
  /// 总令牌数
  pub total_tokens: u32,
  /// 预估成本（美元）
  pub estimated_cost: f64,
}

impl Default for UsageStats {
  fn default() -> Self {
    Self { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0, estimated_cost: 0.0 }
  }
}

/// 流式响应元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingMetadata {
  /// 请求ID
  pub request_id: String,
  /// 模型名称
  pub model: String,
  /// 提供者
  pub provider: String,
  /// 总令牌数
  pub total_tokens: u32,
  /// 是否完成
  pub finished: bool,
  /// 错误信息（如果有）
  pub error: Option<String>,
}
