use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod if_node;
// mod loop_over_items;
// mod merge_node;
// mod set_node;

pub use if_node::IfNode;
// pub use loop_over_items::LoopOverItemsNode;
// pub use merge_node::MergeNode;
// pub use set_node::SetNode;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  node_registry.register_node(IfNode::default())?;
  // node_registry.register_node(MergeNode::default())?;
  // node_registry.register_node(SetNode::default())?;
  // node_registry.register_node(LoopOverItemsNode::default())?;
  Ok(())
}
