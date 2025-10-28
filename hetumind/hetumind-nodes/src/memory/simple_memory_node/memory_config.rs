//! Simplified Memory configuration and data structures for Simple Memory Node
//!
//! 重新设计为基于工作流执行的轻量级内存管理，无需全局状态管理。

use chrono::{DateTime, Utc};
use hetumind_core::types::JsonValue;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Simple Memory 节点配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimpleMemoryConfig {
  /// 会话ID类型
  pub session_id_type: SessionIdType,
  /// 自定义会话密钥（当 session_id_type 为 CustomKey 时使用）
  pub custom_session_key: Option<String>,
  /// 上下文窗口长度（保存的消息数量）
  pub context_window_length: usize,
  /// 输入键名
  pub input_key: String,
  /// 内存键名
  pub memory_key: String,
  /// 输出键名
  pub output_key: String,
  /// 是否返回消息对象
  pub return_messages: bool,
}

impl SimpleMemoryConfig {
  /// 创建默认配置
  pub fn new() -> Self {
    Self {
      session_id_type: SessionIdType::FromInput,
      custom_session_key: None,
      context_window_length: 5,
      input_key: "input".to_string(),
      memory_key: "chat_history".to_string(),
      output_key: "output".to_string(),
      return_messages: true,
    }
  }

  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    if self.context_window_length == 0 {
      return Err("context_window_length must be greater than 0".to_string());
    }

    if self.input_key.is_empty() {
      return Err("input_key cannot be empty".to_string());
    }

    if self.memory_key.is_empty() {
      return Err("memory_key cannot be empty".to_string());
    }

    if self.output_key.is_empty() {
      return Err("output_key cannot be empty".to_string());
    }

    if matches!(self.session_id_type, SessionIdType::CustomKey)
      && self.custom_session_key.as_ref().map_or(true, |s| s.is_empty())
    {
      return Err("custom_session_key is required when session_id_type is CustomKey".to_string());
    }

    Ok(())
  }
}

/// 会话ID类型
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionIdType {
  /// 从输入数据中获取会话ID
  #[default]
  FromInput,
  /// 使用自定义会话密钥
  CustomKey,
}

/// 轻量级内存缓冲区 - 直接在工作流执行中管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMemoryBuffer {
  /// 会话消息历史
  pub messages: VecDeque<ConversationMessage>,
  /// 会话ID
  pub session_id: String,
  /// 最后更新时间
  pub last_updated: DateTime<Utc>,
}

impl WorkflowMemoryBuffer {
  /// 创建新的工作流内存缓冲区
  pub fn new(session_id: String) -> Self {
    Self { messages: VecDeque::new(), session_id, last_updated: Utc::now() }
  }

  /// 添加消息到缓冲区
  pub fn add_message(&mut self, message: ConversationMessage) {
    self.messages.push_back(message);
    self.last_updated = Utc::now();
  }

  /// 获取最近的N条消息（滑动窗口）
  pub fn get_recent_messages(&self, count: usize) -> Vec<ConversationMessage> {
    let buffer_len = self.messages.len();
    if buffer_len <= count {
      self.messages.iter().cloned().collect()
    } else {
      // 返回最近的N条消息
      self.messages.range(buffer_len - count..).cloned().collect()
    }
  }

  /// 获取所有消息
  pub fn get_all_messages(&self) -> Vec<ConversationMessage> {
    self.messages.iter().cloned().collect()
  }

  /// 清空缓冲区
  pub fn clear(&mut self) {
    self.messages.clear();
    self.last_updated = Utc::now();
  }

  /// 获取缓冲区大小
  pub fn len(&self) -> usize {
    self.messages.len()
  }

  /// 检查缓冲区是否为空
  pub fn is_empty(&self) -> bool {
    self.messages.is_empty()
  }
}

/// 会话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
  /// 消息角色
  pub role: MessageRole,
  /// 消息内容
  pub content: String,
  /// 时间戳
  pub timestamp: DateTime<Utc>,
  /// 消息ID（可选）
  pub message_id: Option<String>,
  /// 元数据（可选）
  pub metadata: Option<JsonValue>,
}

impl ConversationMessage {
  /// 创建新的会话消息
  pub fn new(role: MessageRole, content: String) -> Self {
    Self { role, content, timestamp: Utc::now(), message_id: None, metadata: None }
  }

  /// 创建带有ID和元数据的消息
  pub fn with_metadata(
    role: MessageRole,
    content: String,
    message_id: Option<String>,
    metadata: Option<JsonValue>,
  ) -> Self {
    Self { role, content, timestamp: Utc::now(), message_id, metadata }
  }
}

/// 消息角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

/// 内存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
  /// 总消息数量
  pub total_messages: usize,
  /// 会话ID
  pub session_id: String,
  /// 上下文窗口长度
  pub context_window_length: usize,
  /// 统计时间戳
  pub timestamp: DateTime<Utc>,
}

impl MemoryStats {
  /// 创建新的统计信息
  pub fn new(total_messages: usize, session_id: String, context_window_length: usize) -> Self {
    Self { total_messages, session_id, context_window_length, timestamp: Utc::now() }
  }
}
