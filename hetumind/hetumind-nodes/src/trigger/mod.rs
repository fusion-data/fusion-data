mod manual_trigger_node;
mod schedule_trigger_node;
mod start_node;
mod webhook_trigger_node;

pub use manual_trigger_node::*;
pub use schedule_trigger_node::*;
pub use start_node::StartNode;
pub use webhook_trigger_node::*;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

pub fn register_nodes(registry: &NodeRegistry) -> Result<(), RegistrationError> {
  registry.register_node(StartNode::default())?;
  Ok(())
}
