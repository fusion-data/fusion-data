use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{FlowNodeRef, Node, NodeType, RegistrationError, SubNodeRef},
};

mod deepseek_v1;

pub use deepseek_v1::*;

pub struct DeepseekModelNode {
  default_version: Version,
  suppliers: Vec<SubNodeRef>,
}

impl DeepseekModelNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let suppliers: Vec<SubNodeRef> = vec![Arc::new(DeepseekModelV1::new())];
    let default_version = suppliers.iter().map(|node| node.description().version.clone()).max().unwrap();
    Ok(Self { default_version, suppliers })
  }
}

impl Node for DeepseekModelNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[FlowNodeRef] {
    &[]
  }

  fn node_suppliers(&self) -> &[SubNodeRef] {
    &self.suppliers
  }

  fn node_type(&self) -> NodeType {
    self.suppliers[0].description().node_type.clone()
  }
}
