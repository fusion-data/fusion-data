use hetumind_core::{
  version::Version,
  workflow::{Node, NodeExecutor, NodeGroupKind, NodeKind, NodeRegistry, RegistrationError},
};
use std::sync::Arc;

pub mod llm_chat_model_v1;
pub mod parameters;

use llm_chat_model_v1::LlmChatModelV1;

pub struct LlmChatModelNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl LlmChatModelNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let executors: Vec<NodeExecutor> = vec![Arc::new(LlmChatModelV1::new()?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for LlmChatModelNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[NodeExecutor] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let llm_node = Arc::new(LlmChatModelNode::new()?);
  node_registry.register_node(llm_node)?;
  Ok(())
}
