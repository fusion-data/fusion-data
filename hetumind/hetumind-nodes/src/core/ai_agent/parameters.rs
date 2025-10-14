use hetumind_core::types::JsonValue;
use serde::{Deserialize, Serialize};

/// AI Agent 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAgentConfig {
  /// 系统提示词
  pub system_prompt: String,
  /// 最大迭代次数
  pub max_iterations: u32,
  /// 温度参数
  pub temperature: f64,
  /// 是否启用流式响应
  pub enable_streaming: bool,
  /// 记忆配置
  pub memory_config: Option<MemoryConfig>,
}

impl Default for AiAgentConfig {
  fn default() -> Self {
    Self {
      system_prompt: "你是一个有帮助的AI助手".to_string(),
      max_iterations: 10,
      temperature: 0.7,
      enable_streaming: false,
      memory_config: None,
    }
  }
}

/// 记忆配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
  /// 最大历史记录数
  pub max_history: usize,
  /// 是否启用持久化
  pub persistence_enabled: bool,
  /// 上下文窗口大小
  pub context_window: usize,
}

impl Default for MemoryConfig {
  fn default() -> Self {
    Self { max_history: 50, persistence_enabled: false, context_window: 4000 }
  }
}

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
  /// 工具调用ID
  pub id: String,
  /// 工具名称
  pub tool_name: String,
  /// 工具参数
  pub parameters: JsonValue,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
  /// 工具调用ID
  pub tool_call_id: String,
  /// 工具名称
  pub tool_name: String,
  /// 执行结果
  pub result: JsonValue,
  /// 执行状态
  pub status: ToolExecutionStatus,
}

/// 工具执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolExecutionStatus {
  /// 执行成功
  Success,
  /// 执行失败
  Error(String),
  /// 执行超时
  Timeout,
}

/// 会话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
  /// 消息角色
  pub role: MessageRole,
  /// 消息内容
  pub content: String,
  /// 工具调用（如果有）
  pub tool_calls: Option<Vec<ToolCallRequest>>,
  /// 工具结果（如果有）
  pub tool_results: Option<Vec<ToolCallResult>>,
  /// 时间戳
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
  /// 系统消息
  System,
  /// 用户消息
  User,
  /// 助手消息
  Assistant,
  /// 工具消息
  Tool,
}

/// 模型实例信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInstance {
  /// 客户端标识符
  pub client_id: String,
  /// 模型配置
  pub config: JsonValue,
  /// 模型能力
  pub capabilities: Vec<String>,
}
