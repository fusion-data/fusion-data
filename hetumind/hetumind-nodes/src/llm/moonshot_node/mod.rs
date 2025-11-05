use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{FlowNodeRef, Node, NodeKind, NodeRegistry, RegistrationError},
};

mod moonshot_v1;

use moonshot_v1::*;

pub struct MoonshotModelNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl MoonshotModelNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let executors: Vec<FlowNodeRef> = vec![Arc::new(MoonshotV1::new()?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for MoonshotModelNode {
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
  let node = Arc::new(MoonshotModelNode::new()?);
  node_registry.register_node(node)?;
  Ok(())
}
