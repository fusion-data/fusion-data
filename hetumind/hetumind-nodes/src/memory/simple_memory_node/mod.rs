//! Simple Memory Node Implementation
//!
//! 重新设计为基于工作流执行的轻量级内存管理节点。
//! 提供滑动窗口内存管理，支持会话隔离，无需全局状态管理。
//!
//! 主要特性：
//! - 滑动窗口内存管理（固定大小会话历史）
//! - 会话隔离（基于会话ID）
//! - 工作流范围内的内存管理
//! - 简化的数据结构和API

use hetumind_core::{
  version::Version,
  workflow::{Node, FlowNodeRef, NodeKind, NodeRegistry, RegistrationError},
};
use std::sync::Arc;

mod memory_config;
mod simple_memory_v1;

/// Simple Memory 节点常量
pub static SIMPLE_MEMORY_NODE_KIND: &str = "hetumind_nodes::SimpleMemory";

pub struct SimpleMemoryNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl SimpleMemoryNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let executors: Vec<FlowNodeRef> = vec![Arc::new(simple_memory_v1::SimpleMemoryV1::new()?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for SimpleMemoryNode {
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

/// 注册 Simple Memory 节点到节点注册表
pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let simple_memory_node = Arc::new(SimpleMemoryNode::new()?);
  node_registry.register_node(simple_memory_node)?;
  Ok(())
}

// 重新导出主要类型
pub use memory_config::*;
pub use simple_memory_v1::*;

#[cfg(test)]
mod tests {
  use super::*;
  use hetumind_core::workflow::{ConnectionKind, NodeGroupKind};
  use serde_json::json;

  #[test]
  fn test_node_metadata() {
    let node = SimpleMemoryNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), SIMPLE_MEMORY_NODE_KIND);
    assert!(definition.groups.contains(&NodeGroupKind::Transform));
    assert_eq!(&definition.display_name, "Simple Memory");
    assert_eq!(definition.inputs.len(), 1); // AiLM input
    assert_eq!(definition.outputs.len(), 2); // AiMemory + Error output
    assert_eq!(definition.inputs[0].kind, ConnectionKind::AiLM);
    assert_eq!(definition.outputs[0].kind, ConnectionKind::AiMemory);
  }

  #[test]
  fn test_node_registration() {
    let node_registry = NodeRegistry::new();
    assert!(register_nodes(&node_registry).is_ok());

    let registered_node = node_registry.get_executor(&NodeKind::new(SIMPLE_MEMORY_NODE_KIND));
    assert!(registered_node.is_some());
  }

  #[test]
  fn test_memory_buffer_functionality() {
    let mut buffer = WorkflowMemoryBuffer::new("test_session".to_string());

    // 测试添加消息
    let msg1 = ConversationMessage::new(MessageRole::User, "Hello".to_string());
    let msg2 = ConversationMessage::new(MessageRole::Assistant, "Hi there!".to_string());

    buffer.add_message(msg1.clone());
    buffer.add_message(msg2.clone());

    assert_eq!(buffer.len(), 2);

    // 测试滑动窗口
    let recent = buffer.get_recent_messages(1);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].content, "Hi there!");

    // 测试获取所有消息
    let all = buffer.get_all_messages();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].content, "Hello");
    assert_eq!(all[1].content, "Hi there!");
  }

  #[test]
  fn test_simple_memory_accessor() {
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
    assert!(!accessor.is_empty());

    let recent = accessor.get_recent_messages(1);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].content, "Hi there!");
  }
}
