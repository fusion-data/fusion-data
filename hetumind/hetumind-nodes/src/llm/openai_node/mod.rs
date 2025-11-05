use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{FlowNodeRef, Node, NodeKind, NodeRegistry, RegistrationError},
};

mod openai_v1;

use openai_v1::*;

pub struct OpenaiModelNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl OpenaiModelNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let executors: Vec<FlowNodeRef> = vec![Arc::new(OpenaiV1::new()?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for OpenaiModelNode {
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
  let node = Arc::new(OpenaiModelNode::new()?);
  node_registry.register_node(node)?;
  Ok(())
}
