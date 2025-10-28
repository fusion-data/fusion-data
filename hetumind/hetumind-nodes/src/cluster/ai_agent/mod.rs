use hetumind_core::{
  version::Version,
  workflow::{Node, NodeExecutor, NodeKind, NodeRegistry, RegistrationError},
};
use std::sync::Arc;

pub mod ai_agent_v1;
mod graph_flow_agent;
pub mod parameters;
pub mod tool_manager;
mod utils;

use ai_agent_v1::AiAgentV1;
pub use graph_flow_agent::*;

pub struct AiAgentNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl AiAgentNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let executors: Vec<NodeExecutor> = vec![Arc::new(AiAgentV1::new()?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for AiAgentNode {
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
  let ai_agent_node = Arc::new(AiAgentNode::new()?);
  node_registry.register_node(ai_agent_node)?;
  Ok(())
}
