use std::sync::Arc;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod aggregate_node;
mod compare_datasets;
// pub mod connection_manager;
mod edit_fields;
mod edit_image;
mod if_node;
mod limit_node;
mod loop_over_items;
mod merge;
mod no_op_node;
mod read_write_files;
mod send_email;
mod split_out_node;
mod stop_and_error_node;
mod summarize_node;
mod switch;
mod wait_node;

pub use aggregate_node::AggregateNode;
pub use compare_datasets::CompareDatasetsNode;
pub use edit_fields::EditFieldsNode;
pub use edit_image::EditImageNode;
pub use if_node::IfNode;
pub use limit_node::LimitNode;
pub use loop_over_items::LoopOverItemsNode;
pub use merge::MergeNode;
pub use no_op_node::NoOpNode;
pub use read_write_files::ReadWriteFilesNode;
pub use send_email::SendEmailNode;
pub use split_out_node::SplitOutNode;
pub use stop_and_error_node::StopAndErrorNode;
pub use summarize_node::SummarizeNode;
pub use switch::SwitchNode;
pub use wait_node::WaitNode;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let aggregate_node = Arc::new(AggregateNode::new()?);
  node_registry.register_node(aggregate_node)?;

  let compare_datasets_node = Arc::new(CompareDatasetsNode::new()?);
  node_registry.register_node(compare_datasets_node)?;

  let if_node = Arc::new(IfNode::new()?);
  node_registry.register_node(if_node)?;

  let limit_node = Arc::new(LimitNode::new()?);
  node_registry.register_node(limit_node)?;

  let merge_node = Arc::new(MergeNode::new()?);
  node_registry.register_node(merge_node)?;

  let no_op_node = Arc::new(NoOpNode::new()?);
  node_registry.register_node(no_op_node)?;

  let set_node = Arc::new(EditFieldsNode::new()?);
  node_registry.register_node(set_node)?;

  let edit_image_node = Arc::new(EditImageNode::new()?);
  node_registry.register_node(edit_image_node)?;

  let split_out_node = Arc::new(SplitOutNode::new()?);
  node_registry.register_node(split_out_node)?;

  let loop_over_items_node = Arc::new(LoopOverItemsNode::new()?);
  node_registry.register_node(loop_over_items_node)?;

  let read_write_files_node = Arc::new(ReadWriteFilesNode::new()?);
  node_registry.register_node(read_write_files_node)?;

  let send_email_node = Arc::new(SendEmailNode::new()?);
  node_registry.register_node(send_email_node)?;

  let switch_node = Arc::new(SwitchNode::new()?);
  node_registry.register_node(switch_node)?;

  let stop_and_error_node = Arc::new(StopAndErrorNode::new()?);
  node_registry.register_node(stop_and_error_node)?;

  let summarize_node = Arc::new(SummarizeNode::new()?);
  node_registry.register_node(summarize_node)?;

  let wait_node = Arc::new(WaitNode::new()?);
  node_registry.register_node(wait_node)?;

  Ok(())
}
