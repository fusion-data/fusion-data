use hetumind_core::{
  version::Version,
  workflow::{Node, NodeExecutor, NodeKind, NodeRegistry, RegistrationError},
};
use std::sync::Arc;

pub mod chat_model_v1;
pub mod parameters;

use chat_model_v1::ChatModelV1;

pub struct ChatModelNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl ChatModelNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let executors: Vec<NodeExecutor> = vec![Arc::new(ChatModelV1::new()?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for ChatModelNode {
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
  let llm_node = Arc::new(ChatModelNode::new()?);
  node_registry.register_node(llm_node)?;
  Ok(())
}
