//! Simplified Simple Memory Node V1 Implementation
//!
//! 基于工作流执行的轻量级内存管理，无需全局状态。
//! 每个工作流执行实例维护自己的内存缓冲区。

use async_trait::async_trait;
use fusion_core::application::Application;
use hetumind_context::services::memory_service::MemoryService;
use hetumind_core::{
  version::Version,
  workflow::{
    ExecutionDataMap, FlowNode, InputPortConfig, NodeConnectionKind, NodeDescription, NodeExecutionContext,
    NodeExecutionError, NodeGroupKind, NodeProperty, NodePropertyKind, OutputPortConfig, RegistrationError,
  },
};
use log::{debug, info, warn};
use serde_json::json;
use std::sync::Arc;

use crate::constants::SIMPLE_MEMORY_NODE_KIND;
use crate::store::simple_memory_node::memory_config::{
  ConversationMessage, MemoryStats, MessageRole, SessionIdType, SimpleMemoryConfig,
};
use chrono::Utc;

/// Simple Memory Node V1 实现
pub struct SimpleMemoryV1 {
  pub definition: Arc<NodeDescription>,
}

impl SimpleMemoryV1 {
  /// 创建新的 Simple Memory V1 节点
  pub fn new() -> Result<Self, RegistrationError> {
    let definition = Self::create_node_definition();
    Ok(Self { definition: Arc::new(definition) })
  }

  /// 创建节点定义
  fn create_node_definition() -> NodeDescription {
    NodeDescription::new(SIMPLE_MEMORY_NODE_KIND, "Simple Memory")
      .with_version(Version::new(1, 0, 0))
      .with_description(
        "Lightweight in-memory storage for AI workflows with sliding window memory management. \
         Memory is scoped to workflow execution and automatically cleaned up when workflow completes.",
      )
      .add_group(NodeGroupKind::Transform)
      .with_icon("database")
      // Sub-nodes don't have main inputs
      .add_input(InputPortConfig::new(NodeConnectionKind::AiLanguageModel, "LLM连接"))
      // Output ports for memory supply
      .add_output(OutputPortConfig::new(NodeConnectionKind::AiMemory, "内存供应"))
      // Configuration properties
      .add_property(NodeProperty::new(NodePropertyKind::String)
          .with_name("session_id_type")
          .with_display_name("会话ID类型")
          .with_description("如何获取会话ID")
          .with_value(json!("from_input"))
          .with_required(false))
      .add_property(NodeProperty::new(NodePropertyKind::String)
          .with_name("custom_session_key")
          .with_display_name("自定义会话密钥")
          .with_description("当会话ID类型为custom_key时使用的自定义表达式")
          .with_required(false))
      .add_property(NodeProperty::new(NodePropertyKind::Number)
          .with_name("context_window_length")
          .with_display_name("上下文窗口长度")
          .with_description("保存在内存中的最近消息数量")
          .with_value(json!(5))
          .with_required(false))
      .add_property(NodeProperty::new(NodePropertyKind::String)
          .with_name("input_key")
          .with_display_name("输入键名")
          .with_value(json!("input"))
          .with_required(false))
      .add_property(NodeProperty::new(NodePropertyKind::String)
          .with_name("memory_key")
          .with_display_name("内存键名")
          .with_value(json!("chat_history"))
          .with_required(false))
      .add_property(NodeProperty::new(NodePropertyKind::String)
          .with_name("output_key")
          .with_display_name("输出键名")
          .with_value(json!("output"))
          .with_required(false))
      .add_property(NodeProperty::new(NodePropertyKind::Boolean)
          .with_name("return_messages")
          .with_display_name("返回消息对象")
          .with_description("是否返回完整的消息对象而不是仅内容")
          .with_value(json!(true))
          .with_required(false))
  }

  /// 从节点上下文中提取会话ID
  fn extract_session_id(
    &self,
    context: &NodeExecutionContext,
    config: &SimpleMemoryConfig,
  ) -> Result<String, NodeExecutionError> {
    match config.session_id_type {
      SessionIdType::FromInput => {
        // 尝试从输入数据中获取sessionId
        let session_id = context
          .get_input_data(NodeConnectionKind::AiLanguageModel)
          .ok()
          .and_then(|data| data.json().get("session_id").and_then(|v| v.as_str()).map(|s| s.to_string()));

        match session_id {
          Some(id) => {
            debug!("Extracted session ID from input: {}", id);
            Ok(id)
          }
          None => {
            // 如果没有找到sessionId，生成一个默认的
            let default_id = format!("default_session_{}", context.current_node_name());
            warn!("No session ID found in input, using default: {}", default_id);
            Ok(default_id)
          }
        }
      }
      SessionIdType::CustomKey => {
        // 使用自定义会话密钥
        let custom_key = config.custom_session_key.as_ref().ok_or_else(|| {
          NodeExecutionError::ConfigurationError(
            "custom_session_key is required when session_id_type is 'custom_key'".to_string(),
          )
        })?;

        // 这里可以实现更复杂的表达式解析
        // 目前简单地返回自定义密钥
        info!("Using custom session key: {}", custom_key);
        Ok(custom_key.clone())
      }
    }
  }

  /// 从输入数据中提取消息
  fn extract_messages_from_input(
    &self,
    context: &NodeExecutionContext,
  ) -> Result<Vec<ConversationMessage>, NodeExecutionError> {
    let input_data = context.get_input_data(NodeConnectionKind::AiLanguageModel)?;
    let input_json = input_data.json();

    debug!("Extracting messages from input: {}", serde_json::to_string_pretty(input_json).unwrap_or_default());

    // 尝试多种消息格式
    if let Some(messages) = input_json.get("messages").and_then(|v| v.as_array()) {
      return self.parse_message_array(messages);
    }

    if let Some(chat_history) = input_json.get("chat_history").and_then(|v| v.as_array()) {
      return self.parse_message_array(chat_history);
    }

    // 单条消息格式
    if let Some(content) = input_json.get("prompt").and_then(|v| v.as_str()) {
      return Ok(vec![ConversationMessage::new(MessageRole::User, content.to_string())]);
    }

    if let Some(content) = input_json.get("content").and_then(|v| v.as_str()) {
      return Ok(vec![ConversationMessage::new(MessageRole::User, content.to_string())]);
    }

    // 如果没有找到消息，返回空向量而不是错误
    warn!("No messages found in input data");
    Ok(vec![])
  }

  /// 解析消息数组
  fn parse_message_array(
    &self,
    messages: &[serde_json::Value],
  ) -> Result<Vec<ConversationMessage>, NodeExecutionError> {
    let mut conversation_messages = Vec::new();

    for msg in messages {
      let role_str = msg
        .get("role")
        .and_then(|v| v.as_str())
        .ok_or_else(|| NodeExecutionError::InvalidInput("Message missing 'role' field".to_string()))?;

      let content = msg
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| NodeExecutionError::InvalidInput("Message missing 'content' field".to_string()))?;

      let role = match role_str {
        "system" => MessageRole::System,
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "tool" => MessageRole::Tool,
        _ => {
          warn!("Unknown message role: {}, defaulting to 'user'", role_str);
          MessageRole::User
        }
      };

      let mut message = ConversationMessage::new(role, content.to_string());

      // 提取可选的message_id
      if let Some(message_id) = msg.get("id").and_then(|v| v.as_str()) {
        message.message_id = Some(message_id.to_string());
      }

      // 提取可选的元数据
      if let Some(metadata) = msg.get("metadata") {
        message.metadata = Some(metadata.clone());
      }

      conversation_messages.push(message);
    }

    Ok(conversation_messages)
  }

  // 已弃用：SimpleMemoryV1 现已改为使用 MemoryService 组件进行会话内存存储与检索
}

impl TryFrom<NodeDescription> for SimpleMemoryV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDescription) -> Result<Self, Self::Error> {
    let definition = base.with_version(Version::new(1, 0, 0));
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl FlowNode for SimpleMemoryV1 {
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // 1. 提取配置
    let config: SimpleMemoryConfig = context.get_parameters()?;

    // 验证配置
    config
      .validate()
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("Invalid SimpleMemory configuration: {}", e)))?;

    debug!("SimpleMemory config: {:?}", config);

    // 2. 获取工作流ID
    let workflow_id = context.workflow.id.clone();

    // 3. 提取会话ID
    let session_id = self.extract_session_id(context, &config)?;
    info!("Using session ID: {} for workflow: {}", session_id, workflow_id);

    // 4. 从输入中提取消息
    let input_messages = self.extract_messages_from_input(context)?;

    // 5. 从全局 Application 中获取 MemoryService 组件
    let svc = Application::global()
      .get_component::<Arc<dyn MemoryService>>()
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("MemoryService component not found: {}", e)))?;

    // 6. 保存输入消息到会话内存（通过 MemoryService）
    // 将 ConversationMessage 转换为通用 Message 类型
    let to_core_msgs: Vec<hetumind_core::workflow::Message> = input_messages
      .into_iter()
      .map(|m| hetumind_core::workflow::Message {
        role: match m.role {
          MessageRole::System => "system".to_string(),
          MessageRole::User => "user".to_string(),
          MessageRole::Assistant => "assistant".to_string(),
          MessageRole::Tool => "tool".to_string(),
        },
        content: m.content,
      })
      .collect();

    let tenant_id = "default_tenant"; // TODO: 从执行上下文注入真实租户ID
    let workflow_id_string = context.workflow.id.to_string();
    let workflow_id = workflow_id_string.as_str();
    svc.store_messages(tenant_id, workflow_id, &session_id, to_core_msgs)?;

    // 7. 获取最近的N条消息（滑动窗口）
    let recent_messages_core =
      svc.retrieve_messages(tenant_id, workflow_id, &session_id, config.context_window_length)?;
    debug!("Retrieved {} recent messages from memory service: {}", recent_messages_core.len(), session_id);

    // 将通用 Message 映射到输出结构（补充 timestamp 字段以便UI展示）
    let recent_messages: Vec<ConversationMessage> = recent_messages_core
      .into_iter()
      .map(|m| {
        let role = match m.role.as_str() {
          "system" => MessageRole::System,
          "assistant" => MessageRole::Assistant,
          "tool" => MessageRole::Tool,
          _ => MessageRole::User,
        };
        let mut cm = ConversationMessage::new(role, m.content);
        // 使用当前时间作为展示用途的时间戳（MemoryService 不保存逐条消息时间戳）
        cm.timestamp = Utc::now();
        cm
      })
      .collect();

    // 8. 创建内存统计信息（使用 get_buffer.len() 作为总消息数）
    let total_messages = svc.get_buffer(tenant_id, workflow_id, &session_id)?.len();
    let memory_stats = MemoryStats::new(total_messages, session_id.clone(), config.context_window_length);

    // 9. 创建执行数据映射
    let mut data_map = ExecutionDataMap::default();

    // 添加内存供应数据
    let memory_supply_data = json!({
      "session_id": session_id,
      "workflow_id": workflow_id,
      "config": config,
      "chat_history": recent_messages.iter().map(|msg| {
        json!({
          "role": msg.role,
          "content": msg.content,
          "timestamp": msg.timestamp.to_rfc3339(),
          "message_id": msg.message_id,
          "metadata": msg.metadata,
      })
    }).collect::<Vec<_>>(),
      "stats": memory_stats,
    });

    data_map.insert(
      NodeConnectionKind::AiMemory,
      vec![hetumind_core::workflow::ExecutionDataItems::new_item(
        hetumind_core::workflow::ExecutionData::new_json(
          memory_supply_data,
          Some(hetumind_core::workflow::DataSource::new(
            context.current_node_name().clone(),
            NodeConnectionKind::AiMemory,
            0,
          )),
        ),
      )],
    );

    info!("SimpleMemory node executed successfully");
    debug!("Memory created with {} messages", recent_messages.len());

    Ok(data_map)
  }

  fn description(&self) -> Arc<NodeDescription> {
    Arc::clone(&self.definition)
  }
}

/// 简化的内存访问接口
/// 可以被 AiAgent 等其他节点使用来访问 SimpleMemory 的数据
#[derive(Debug, Clone)]
pub struct SimpleMemoryAccessor {
  pub session_id: String,
  pub workflow_id: String,
  pub chat_history: Vec<ConversationMessage>,
  pub stats: MemoryStats,
}

impl SimpleMemoryAccessor {
  /// 从 SimpleMemory 的输出数据创建访问器
  pub fn from_execution_data(
    execution_data: &hetumind_core::workflow::ExecutionData,
  ) -> Result<Self, NodeExecutionError> {
    let data = execution_data.json();

    let session_id = data
      .get("session_id")
      .and_then(|v| v.as_str())
      .ok_or_else(|| NodeExecutionError::InvalidInput("Missing session_id in memory data".to_string()))?
      .to_string();

    let workflow_id = data
      .get("workflow_id")
      .and_then(|v| v.as_str())
      .ok_or_else(|| NodeExecutionError::InvalidInput("Missing workflow_id in memory data".to_string()))?
      .to_string();

    // 解析聊天历史
    let chat_history: Vec<ConversationMessage> = data
      .get("chat_history")
      .and_then(|v| v.as_array())
      .map(|arr| {
        arr
          .iter()
          .filter_map(|msg| {
            let role_str = msg.get("role")?.as_str()?;
            let content = msg.get("content")?.as_str()?;
            let timestamp_str = msg.get("timestamp")?.as_str()?;

            let role = match role_str {
              "system" => MessageRole::System,
              "user" => MessageRole::User,
              "assistant" => MessageRole::Assistant,
              "tool" => MessageRole::Tool,
              _ => return None,
            };

            let timestamp = timestamp_str.parse().ok()?;

            let mut conversation_msg = ConversationMessage::new(role, content.to_string());
            conversation_msg.timestamp = timestamp;

            if let Some(message_id) = msg.get("message_id").and_then(|v| v.as_str()) {
              conversation_msg.message_id = Some(message_id.to_string());
            }

            if let Some(metadata) = msg.get("metadata") {
              conversation_msg.metadata = Some(metadata.clone());
            }

            Some(conversation_msg)
          })
          .collect()
      })
      .unwrap_or_default();

    // 解析统计信息
    let stats_data = data
      .get("stats")
      .ok_or_else(|| NodeExecutionError::InvalidInput("Missing stats in memory data".to_string()))?;

    let total_messages = stats_data.get("total_messages").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

    let context_window_length = stats_data.get("context_window_length").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

    let stats = MemoryStats::new(total_messages, session_id.clone(), context_window_length);

    Ok(Self { session_id, workflow_id, chat_history, stats })
  }

  /// 获取最近的N条消息
  pub fn get_recent_messages(&self, count: usize) -> Vec<&ConversationMessage> {
    let len = self.chat_history.len();
    if len <= count { self.chat_history.iter().collect() } else { self.chat_history.iter().skip(len - count).collect() }
  }

  /// 获取所有消息
  pub fn get_all_messages(&self) -> &Vec<ConversationMessage> {
    &self.chat_history
  }

  /// 获取消息数量
  pub fn len(&self) -> usize {
    self.chat_history.len()
  }

  /// 检查是否为空
  pub fn is_empty(&self) -> bool {
    self.chat_history.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // 引入 WorkflowMemoryBuffer 以便测试本地缓冲区逻辑
  use crate::store::WorkflowMemoryBuffer;

  #[tokio::test]
  async fn test_node_creation() {
    let node = SimpleMemoryV1::new().unwrap();
    let definition = node.description();

    assert_eq!(definition.node_type.as_ref(), SIMPLE_MEMORY_NODE_KIND);
    assert_eq!(definition.display_name, "Simple Memory");
    assert_eq!(definition.version, Version::new(1, 0, 0));
  }

  #[tokio::test]
  async fn test_message_extraction() {
    let node = SimpleMemoryV1::new().unwrap();

    // 测试从消息数组提取
    let messages_json = json!({
      "messages": [
        {"role": "user", "content": "Hello"},
        {"role": "assistant", "content": "Hi there!"}
      ]
    });

    // 测试解析逻辑
    let messages = node.parse_message_array(messages_json.get("messages").unwrap().as_array().unwrap()).unwrap();

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, MessageRole::User);
    assert_eq!(messages[0].content, "Hello");
    assert_eq!(messages[1].role, MessageRole::Assistant);
    assert_eq!(messages[1].content, "Hi there!");
  }

  #[tokio::test]
  async fn test_workflow_memory_buffer() {
    let mut buffer = WorkflowMemoryBuffer::new("test_session".to_string());

    // 添加消息
    let msg1 = ConversationMessage::new(MessageRole::User, "Hello".to_string());
    let msg2 = ConversationMessage::new(MessageRole::Assistant, "Hi there!".to_string());
    let msg3 = ConversationMessage::new(MessageRole::User, "How are you?".to_string());

    buffer.add_message(msg1);
    buffer.add_message(msg2);
    buffer.add_message(msg3);

    assert_eq!(buffer.len(), 3);

    // 测试滑动窗口
    let recent = buffer.get_recent_messages(2);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].content, "Hi there!");
    assert_eq!(recent[1].content, "How are you?");

    // 清空缓冲区
    buffer.clear();
    assert!(buffer.is_empty());
  }

  #[test]
  fn test_config_validation() {
    let mut config = SimpleMemoryConfig::new();

    // 有效配置应该通过验证
    assert!(config.validate().is_ok());

    // 无效的上下文窗口长度
    config.context_window_length = 0;
    assert!(config.validate().is_err());

    // 恢复有效值
    config.context_window_length = 5;
    assert!(config.validate().is_ok());

    // 自定义会话密钥但没有提供值
    config.session_id_type = SessionIdType::CustomKey;
    assert!(config.validate().is_err());

    // 提供自定义会话密钥
    config.custom_session_key = Some("custom_session".to_string());
    assert!(config.validate().is_ok());
  }

  #[test]
  fn test_memory_accessor() {
    // 创建模拟的执行数据
    let memory_data = json!({
      "session_id": "test_session",
      "workflow_id": "test_workflow",
      "chat_history": [
        {
          "role": "user",
          "content": "Hello",
          "timestamp": "2024-01-01T00:00:00Z"
        },
        {
          "role": "assistant",
          "content": "Hi there!",
          "timestamp": "2024-01-01T00:00:01Z"
        }
      ],
      "stats": {
        "total_messages": 2,
        "context_window_length": 5
      }
    });

    let execution_data = hetumind_core::workflow::ExecutionData::new_json(memory_data, None);
    let accessor = SimpleMemoryAccessor::from_execution_data(&execution_data).unwrap();

    assert_eq!(accessor.session_id, "test_session");
    assert_eq!(accessor.workflow_id, "test_workflow");
    assert_eq!(accessor.len(), 2);

    let recent = accessor.get_recent_messages(1);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].content, "Hi there!");
  }
}
