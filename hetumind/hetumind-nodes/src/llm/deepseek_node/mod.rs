use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeExecutor, NodeKind, NodeRegistry, RegistrationError},
};

mod deepseek_v1;
mod graph_flow_deepseek;

use deepseek_v1::*;
pub use graph_flow_deepseek::*;

pub struct DeepseekModelNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl DeepseekModelNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let executors: Vec<NodeExecutor> = vec![Arc::new(DeepseekV1::new()?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for DeepseekModelNode {
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
  let ai_agent_node = Arc::new(DeepseekModelNode::new()?);
  node_registry.register_node(ai_agent_node)?;
  Ok(())
}
