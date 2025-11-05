use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{FlowNodeRef, Node, NodeKind, NodeRegistry, RegistrationError, SubNodeRef},
};

mod deepseek_v1;
mod supplier;

pub use deepseek_v1::*;
pub use supplier::*;

pub struct DeepseekModelNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
  suppliers: Vec<SubNodeRef>,
}

impl DeepseekModelNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let executors: Vec<FlowNodeRef> = vec![Arc::new(DeepseekV1::new()?)];
    let suppliers: Vec<SubNodeRef> = vec![Arc::new(DeepseekModelSupplier::new())];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors, suppliers })
  }
}

impl Node for DeepseekModelNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[FlowNodeRef] {
    &self.executors
  }

  fn node_suppliers(&self) -> &[SubNodeRef] {
    &self.suppliers
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let deepseek_node = Arc::new(DeepseekModelNode::new()?);
  node_registry.register_node(deepseek_node.clone())?;
  for s in deepseek_node.node_suppliers() {
    // 同时注册 typed LLM Supplier
    node_registry.register_subnode_provider(deepseek_node.kind(), s.clone())?;
  }
  // 额外注册一个 typed LLM Supplier（便于类型安全的获取），与 Node 内的 SubNodeRef 保持功能一致
  let typed_supplier: hetumind_core::workflow::LLMSubNodeProviderRef = Arc::new(DeepseekModelSupplier::new());
  node_registry.register_llm_supplier(deepseek_node.kind(), typed_supplier)?;
  Ok(())
}
