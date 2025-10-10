mod schedule_trigger;
mod start;
mod webhook_trigger;

use std::sync::Arc;

pub use schedule_trigger::ScheduleTriggerNode;
pub use start::StartNode;
pub use webhook_trigger::WebhookTriggerNode;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

pub fn register_nodes(registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let node = StartNode::new()?;
  registry.register_node(Arc::new(node))?;

  let webhook_node = WebhookTriggerNode::new()?;
  registry.register_node(Arc::new(webhook_node))?;

  let schedule_node = ScheduleTriggerNode::new()?;
  registry.register_node(Arc::new(schedule_node))?;

  Ok(())
}
