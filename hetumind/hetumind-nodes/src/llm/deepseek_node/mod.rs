use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, FlowNodeRef, NodeKind, NodeRegistry, RegistrationError},
};

mod deepseek_v1;

use deepseek_v1::*;

pub struct DeepseekModelNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl DeepseekModelNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let executors: Vec<FlowNodeRef> = vec![Arc::new(DeepseekV1::new()?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for DeepseekModelNode {
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

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let ai_agent_node = Arc::new(DeepseekModelNode::new()?);
  node_registry.register_node(ai_agent_node)?;
  Ok(())
}
