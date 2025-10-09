use std::sync::Arc;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod if_node;
mod loop_over_items;
mod merge_node;
mod set_node;

pub use if_node::IfNode;
pub use loop_over_items::LoopOverItemsNode;
pub use merge_node::MergeNode;
pub use set_node::SetNode;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let if_node = Arc::new(IfNode::new()?);
  node_registry.register_node(if_node)?;

  let set_node = Arc::new(SetNode::new()?);
  node_registry.register_node(set_node)?;

  let merge_node = Arc::new(MergeNode::new()?);
  node_registry.register_node(merge_node)?;

  let loop_over_items_node = Arc::new(LoopOverItemsNode::new()?);
  node_registry.register_node(loop_over_items_node)?;

  Ok(())
}
