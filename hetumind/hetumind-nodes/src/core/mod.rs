use std::sync::Arc;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod aggregate_node;
mod ai_agent;
mod connection_manager;
mod if_node;
mod limit_node;
mod llm_chat_model;
mod loop_over_items;
mod merge;
mod no_op_node;
mod read_write_files;
mod set;
mod split_out_node;
mod stop_and_error_node;
mod summarize_node;
mod switch;

pub use aggregate_node::AggregateNode;
pub use if_node::IfNode;
pub use limit_node::LimitNode;
pub use loop_over_items::LoopOverItemsNode;
pub use merge::MergeNode;
pub use no_op_node::NoOpNode;
pub use read_write_files::ReadWriteFilesNode;
pub use set::SetNode;
pub use split_out_node::SplitOutNode;
pub use stop_and_error_node::StopAndErrorNode;
pub use summarize_node::SummarizeNode;
pub use switch::SwitchNode;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let aggregate_node = Arc::new(AggregateNode::new()?);
  node_registry.register_node(aggregate_node)?;

  let if_node = Arc::new(IfNode::new()?);
  node_registry.register_node(if_node)?;

  let limit_node = Arc::new(LimitNode::new()?);
  node_registry.register_node(limit_node)?;

  let merge_node = Arc::new(MergeNode::new()?);
  node_registry.register_node(merge_node)?;

  let no_op_node = Arc::new(NoOpNode::new()?);
  node_registry.register_node(no_op_node)?;

  let set_node = Arc::new(SetNode::new()?);
  node_registry.register_node(set_node)?;

  let split_out_node = Arc::new(SplitOutNode::new()?);
  node_registry.register_node(split_out_node)?;

  let loop_over_items_node = Arc::new(LoopOverItemsNode::new()?);
  node_registry.register_node(loop_over_items_node)?;

  let read_write_files_node = Arc::new(ReadWriteFilesNode::new()?);
  node_registry.register_node(read_write_files_node)?;

  let switch_node = Arc::new(SwitchNode::new()?);
  node_registry.register_node(switch_node)?;

  let stop_and_error_node = Arc::new(StopAndErrorNode::new()?);
  node_registry.register_node(stop_and_error_node)?;

  let summarize_node = Arc::new(SummarizeNode::new()?);
  node_registry.register_node(summarize_node)?;

  // Register AI nodes
  ai_agent::register_nodes(node_registry)?;
  llm_chat_model::register_nodes(node_registry)?;

  Ok(())
}
