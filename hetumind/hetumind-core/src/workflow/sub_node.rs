//! Sub Node Provider 核心接口定义
//!
//! 此模块定义了 Cluster Node 架构中的核心接口，支持 LLM、Memory、Tool 等不同类型的 Sub Node Provider。

use crate::workflow::{NodeDefinition, NodeExecutionError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Sub Node Provider 类型别名，统一使用 Arc 包装
pub type SubNodeRef = Arc<dyn SubNode + Sync + Send>;
pub type LLMSubNodeProviderRef = Arc<dyn LLMSubNodeProvider + Sync + Send>;
pub type MemorySubNodeProviderRef = Arc<dyn MemorySubNodeProvider + Sync + Send>;
pub type ToolSubNodeProviderRef = Arc<dyn ToolSubNodeProvider + Sync + Send>;
pub type AgentSubNodeProviderRef = Arc<dyn AgentSubNodeProvider + Sync + Send>;

/// Sub Node Provider 类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubNodeType {
  LLM,
  Memory,
  Tool,
  Agent,
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
  pub model: String,
  pub max_tokens: Option<u32>,
  pub temperature: Option<f64>,
  pub top_p: Option<u32>,
  pub stop_sequences: Option<Vec<String>>,
  pub api_key: Option<String>,
}

/// LLM 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
  pub content: String,
  pub role: String,
  pub usage: Option<UsageStats>,
}

/// 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
  pub prompt_tokens: u32,
  pub completion_tokens: u32,
  pub total_tokens: u32,
}

/// Memory 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
  pub context_window: Option<usize>,
  pub max_history: Option<usize>,
  pub persistence_enabled: Option<bool>,
}

/// Tool 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
  pub name: String,
  pub description: String,
  pub parameters: serde_json::Value,
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
  /// 系统提示词
  pub system_prompt: Option<String>,
  /// 最大迭代次数
  pub max_iterations: Option<u32>,
  /// 温度参数
  pub temperature: Option<f64>,
  /// 是否启用流式响应
  pub enable_streaming: Option<bool>,
  /// 是否启用工具调用
  pub enable_tools: Option<bool>,
  /// 会话ID
  pub session_id: Option<String>,
}

/// Agent 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
  /// 响应内容
  pub content: String,
  /// 响应角色
  pub role: String,
  /// 工具调用（如果有）
  pub tool_calls: Option<Vec<ToolCallRequest>>,
  /// 使用统计
  pub usage: Option<AgentUsageStats>,
  /// 会话信息
  pub session_info: Option<SessionInfo>,
}

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
  /// 工具调用ID
  pub id: String,
  /// 工具名称
  pub tool_name: String,
  /// 工具参数
  pub parameters: serde_json::Value,
}

/// Agent 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageStats {
  /// 总迭代次数
  pub total_iterations: u32,
  /// LLM 调用次数
  pub llm_calls: u32,
  /// 工具调用次数
  pub tool_calls: u32,
  /// 总令牌数（如果可用）
  pub total_tokens: Option<u32>,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
  /// 会话ID
  pub session_id: String,
  /// 历史消息数
  pub history_length: usize,
  /// 是否使用内存
  pub has_memory: bool,
}

/// Sub Node Provider 基础 trait
#[async_trait]
pub trait SubNode: Send + Sync {
  /// 获取 Provider 类型
  fn provider_type(&self) -> SubNodeType;

  /// 获取节点定义
  fn definition(&self) -> Arc<NodeDefinition>;

  /// 初始化 Provider
  async fn initialize(&self) -> Result<(), NodeExecutionError>;
}

/// Message placeholder type for LLM interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  pub role: String,
  pub content: String,
}

/// LLM Sub Node Provider 接口
#[async_trait]
pub trait LLMSubNodeProvider: SubNode {
  /// 调用 LLM
  async fn call_llm(&self, messages: Vec<Message>, config: LLMConfig) -> Result<LLMResponse, NodeExecutionError>;
}

/// Memory Sub Node Provider 接口
#[async_trait]
pub trait MemorySubNodeProvider: SubNode {
  /// 存储消息
  async fn store_messages(&self, session_id: &str, messages: Vec<Message>) -> Result<(), NodeExecutionError>;

  /// 检索消息
  async fn retrieve_messages(&self, session_id: &str, count: usize) -> Result<Vec<Message>, NodeExecutionError>;
}

/// Tool placeholder type for tool interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
  pub name: String,
  pub description: String,
}

/// Tool Sub Node Provider 接口
#[async_trait]
pub trait ToolSubNodeProvider: SubNode {
  /// 转换为工具
  async fn as_tool(&self) -> Result<Tool, NodeExecutionError>;
}

/// Agent Sub Node Provider 接口
#[async_trait]
pub trait AgentSubNodeProvider: SubNode {
  /// 执行 Agent 任务
  async fn execute_agent(
    &self,
    messages: Vec<Message>,
    config: AgentConfig,
  ) -> Result<AgentResponse, NodeExecutionError>;
}

/// Cluster Node 执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNodeConfig {
  /// LLM 配置
  pub llm_config: Option<LLMConfig>,
  /// Memory 配置
  pub memory_config: Option<MemoryConfig>,
  /// Tool 配置
  pub tools_config: Option<Vec<ToolConfig>>,
  /// Agent 配置
  pub agent_config: Option<AgentConfig>,
  /// 执行配置
  pub execution_config: ExecutionConfig,
}

/// 执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
  pub timeout_seconds: Option<u64>,
  pub max_retries: Option<u32>,
  pub parallel_execution: Option<bool>,
}

impl Default for ClusterNodeConfig {
  fn default() -> Self {
    Self {
      llm_config: None,
      memory_config: None,
      tools_config: None,
      agent_config: None,
      execution_config: ExecutionConfig {
        timeout_seconds: Some(30),
        max_retries: Some(3),
        parallel_execution: Some(true),
      },
    }
  }
}

impl Default for ExecutionConfig {
  fn default() -> Self {
    Self { timeout_seconds: Some(30), max_retries: Some(3), parallel_execution: Some(true) }
  }
}

impl Default for LLMConfig {
  fn default() -> Self {
    Self {
      model: "default".to_string(),
      max_tokens: None,
      temperature: Some(0.7),
      top_p: None,
      stop_sequences: None,
      api_key: None,
    }
  }
}

impl Default for MemoryConfig {
  fn default() -> Self {
    Self { context_window: Some(5), max_history: Some(100), persistence_enabled: Some(false) }
  }
}

impl Default for ToolConfig {
  fn default() -> Self {
    Self {
      name: "default_tool".to_string(),
      description: "Default tool".to_string(),
      parameters: serde_json::Value::Null,
    }
  }
}

impl Default for AgentConfig {
  fn default() -> Self {
    Self {
      system_prompt: Some("You are a helpful AI assistant.".to_string()),
      max_iterations: Some(10),
      temperature: Some(0.7),
      enable_streaming: Some(false),
      enable_tools: Some(true),
      session_id: None,
    }
  }
}
