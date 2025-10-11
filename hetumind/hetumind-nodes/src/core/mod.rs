use std::sync::Arc;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod ai_agent;
mod connection_manager;
mod if_node;
mod llm_chat_model;
mod loop_over_items;
mod merge;
mod read_write_files;
mod set;
mod switch;

pub use if_node::IfNode;
pub use loop_over_items::LoopOverItemsNode;
pub use merge::MergeNode;
pub use read_write_files::ReadWriteFilesNode;
pub use set::SetNode;
pub use switch::SwitchNode;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let if_node = Arc::new(IfNode::new()?);
  node_registry.register_node(if_node)?;

  let set_node = Arc::new(SetNode::new()?);
  node_registry.register_node(set_node)?;

  let merge_node = Arc::new(MergeNode::new()?);
  node_registry.register_node(merge_node)?;

  let loop_over_items_node = Arc::new(LoopOverItemsNode::new()?);
  node_registry.register_node(loop_over_items_node)?;

  let read_write_files_node = Arc::new(ReadWriteFilesNode::new()?);
  node_registry.register_node(read_write_files_node)?;

  let switch_node = Arc::new(SwitchNode::new()?);
  node_registry.register_node(switch_node)?;

  // Register AI nodes
  ai_agent::register_nodes(node_registry)?;
  llm_chat_model::register_nodes(node_registry)?;

  Ok(())
}
