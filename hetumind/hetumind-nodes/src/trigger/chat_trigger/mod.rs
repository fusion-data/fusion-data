//! # ChatTriggerNode
//!
//! A node that triggers workflow execution when receiving chat messages.

use std::sync::Arc;

use ahash::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use fusion_common::time::now_offset;
use hetumind_core::types::JsonValue;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, Node, NodeDefinition, FlowNode,
  NodeExecutionContext, NodeExecutionError, FlowNodeRef, NodeGroupKind, NodeKind, NodeProperty, NodePropertyKind,
  RegistrationError, make_execution_data_map,
};

use crate::constants::CHAT_TRIGGERN_NODE_KIND as CHAT_TRIGGER_NODE_KIND;
use serde_json::json;

// 聊天消息结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
  pub chat_id: String,    // 聊天会话ID
  pub message_id: String, // 消息唯一ID
  pub user_id: String,    // 用户ID
  pub content: String,    // 消息内容
  pub timestamp: DateTime<FixedOffset>,
  pub session_data: HashMap<String, JsonValue>, // 会话数据
  pub metadata: HashMap<String, JsonValue>,     // 自定义元数据
}

// 聊天接口类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ChatInterfaceType {
  Web,            // Web聊天界面
  Embedded,       // 嵌入式聊天
  Api,            // API接口
  Custom(String), // 自定义接口
}

// 会话存储方式
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SessionStorage {
  Memory,   // 内存存储
  Redis,    // Redis存储
  Database, // 数据库存储
  File,     // 文件存储
}

// 认证方式
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthenticationMethod {
  None,                                         // 无认证
  ApiKey { key: String },                       // API密钥
  Bearer { token: String },                     // Bearer令牌
  Basic { username: String, password: String }, // 基本认证
  OAuth2,                                       // OAuth2认证
}

pub struct ChatTriggerNodeV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinition> for ChatTriggerNodeV1 {
  type Error = RegistrationError;

  fn try_from(definition: NodeDefinition) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl FlowNode for ChatTriggerNodeV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;

    // 解析节点参数
    let chat_interface: String =
      node.parameters.get_optional_parameter("chatInterface").unwrap_or_else(|| "web".to_string());
    let session_timeout: u32 = node.parameters.get_optional_parameter("sessionTimeout").unwrap_or(30);
    let response_format: String =
      node.parameters.get_optional_parameter("responseFormat").unwrap_or_else(|| "json".to_string());

    // 构建聊天消息数据结构
    let chat_id = uuid::Uuid::now_v7().to_string();
    let message_id = uuid::Uuid::now_v7().to_string();
    let timestamp = now_offset();

    let chat_message = ChatMessage {
      chat_id: chat_id.clone(),
      message_id,
      user_id: "anonymous".to_string(),               // 实际应从请求中获取
      content: "Hello from chat trigger".to_string(), // 实际应从请求中获取
      timestamp,
      session_data: [
        ("chatInterface".to_string(), json!(chat_interface)),
        ("sessionTimeout".to_string(), json!(session_timeout)),
        ("responseFormat".to_string(), json!(response_format)),
      ]
      .into_iter()
      .collect(),
      metadata: HashMap::default(), // 实际应从请求中获取元数据
    };

    // 将聊天消息转换为 JSON
    let message_json = serde_json::to_value(chat_message).map_err(|e| NodeExecutionError::DataProcessingError {
      message: format!("Failed to serialize chat message: {}", e),
    })?;

    // 创建包含聊天消息的执行数据
    let execution_data = ExecutionData::new_json(message_json, None);
    let execution_data_items = ExecutionDataItems::new_items(vec![execution_data]);

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![execution_data_items])]))
  }
}

pub struct ChatTriggerNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl Node for ChatTriggerNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[FlowNodeRef] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

impl ChatTriggerNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_base();
    let executors: Vec<FlowNodeRef> = vec![Arc::new(ChatTriggerNodeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

fn create_base() -> NodeDefinition {
  NodeDefinition::new(CHAT_TRIGGER_NODE_KIND, "Chat Trigger")
    .add_group(NodeGroupKind::Trigger)
    .with_description("Use the Chat Trigger node when building AI workflows for chatbots and other chat interfaces.")
    .add_property(
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("Chat Interface")
        .with_name("chatInterface")
        .with_options(vec![
          Box::new(NodeProperty::new_option("Web", "web", json!("web"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("Embedded", "embedded", json!("embedded"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("API", "api", json!("api"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("Custom", "custom", json!("custom"), NodePropertyKind::String)),
        ])
        .with_required(true)
        .with_description("The type of chat interface to use"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Number)
        .with_display_name("Session Timeout (minutes)")
        .with_name("sessionTimeout")
        .with_required(false)
        .with_description("Session timeout in minutes")
        .with_value(json!(30)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Number)
        .with_display_name("Max Session Length")
        .with_name("maxSessionLength")
        .with_required(false)
        .with_description("Maximum number of messages in a session")
        .with_value(json!(50)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Boolean)
        .with_display_name("Persist Session")
        .with_name("persistSession")
        .with_required(false)
        .with_description("Whether to persist session data")
        .with_value(json!(true)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("Response Format")
        .with_name("responseFormat")
        .with_options(vec![
          Box::new(NodeProperty::new_option("JSON", "json", json!("json"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("Text", "text", json!("text"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("Markdown", "markdown", json!("markdown"), NodePropertyKind::String)),
        ])
        .with_required(true)
        .with_description("Format of the response"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Boolean)
        .with_display_name("Immediate Response")
        .with_name("immediateResponse")
        .with_required(false)
        .with_description("Whether to send an immediate response")
        .with_value(json!(false)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::String)
        .with_display_name("Response Template")
        .with_name("responseTemplate")
        .with_required(false)
        .with_description("Template for immediate responses")
        .with_value(json!({"status": "received", "chatId": "{{chatId}}"})),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("Authentication")
        .with_name("authentication")
        .with_options(vec![
          Box::new(NodeProperty::new_option("None", "none", json!("none"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("API Key", "api_key", json!("api_key"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("Bearer Token", "bearer", json!("bearer"), NodePropertyKind::String)),
        ])
        .with_required(true)
        .with_description("Authentication method for the chat interface"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Boolean)
        .with_display_name("Include Metadata")
        .with_name("includeMetadata")
        .with_required(false)
        .with_description("Whether to include metadata in the output")
        .with_value(json!(true)),
    )
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use hetumind_core::workflow::NodeRegistry;

  use super::*;

  #[test]
  fn test_chat_trigger_node_registration() {
    let registry = NodeRegistry::new();
    let node = ChatTriggerNode::new().unwrap();
    let node_kind = node.kind();
    registry.register_node(Arc::new(node)).unwrap();

    // 验证节点已注册
    assert!(registry.contains(&node_kind));
  }

  #[test]
  fn test_chat_trigger_node_definition() {
    let node = ChatTriggerNode::new().unwrap();
    let executor = &node.node_executors()[0];
    let definition = executor.definition();

    assert_eq!(definition.display_name.as_str(), "Chat Trigger");
    assert!(definition.description.as_deref().is_some_and(|s| s.contains("chatbots")));
    assert_eq!(definition.groups, vec![NodeGroupKind::Trigger]);
  }

  #[tokio::test]
  async fn test_chat_message_structure() {
    let chat_message = ChatMessage {
      chat_id: "test-chat-id".to_string(),
      message_id: "test-message-id".to_string(),
      user_id: "test-user".to_string(),
      content: "Hello, world!".to_string(),
      timestamp: now_offset(),
      session_data: HashMap::default(),
      metadata: HashMap::default(),
    };

    // 验证消息结构可以正确序列化
    let json = serde_json::to_value(&chat_message).unwrap();
    assert_eq!(json["chat_id"], "test-chat-id");
    assert_eq!(json["content"], "Hello, world!");
    assert_eq!(json["message_id"], "test-message-id");
    assert_eq!(json["user_id"], "test-user");
  }
}
