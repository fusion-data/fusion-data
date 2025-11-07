use std::sync::Arc;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod if_node;
mod loop_over_items;
mod no_op_node;

pub use if_node::IfNode;
pub use loop_over_items::LoopOverItemsNode;
pub use no_op_node::NoOpNode;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let if_node = Arc::new(IfNode::new()?);
  node_registry.register_node(if_node)?;

  let no_op_node = Arc::new(NoOpNode::new()?);
  node_registry.register_node(no_op_node)?;

  let loop_over_items_node = Arc::new(LoopOverItemsNode::new()?);
  node_registry.register_node(loop_over_items_node)?;

  Ok(())
}
