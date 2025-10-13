mod chat_trigger;
mod email_trigger;
mod error_trigger;
mod manual_trigger;
mod schedule_trigger;
mod start;
mod webhook_trigger;

use std::sync::Arc;

pub use chat_trigger::ChatTriggerNode;
pub use email_trigger::EmailTriggerNode;
pub use error_trigger::ErrorTriggerNode;
pub use manual_trigger::ManualTriggerNode;
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

  let chat_node = ChatTriggerNode::new()?;
  registry.register_node(Arc::new(chat_node))?;

  let error_node = ErrorTriggerNode::new()?;
  registry.register_node(Arc::new(error_node))?;

  let manual_trigger_node = ManualTriggerNode::new()?;
  registry.register_node(Arc::new(manual_trigger_node))?;

  let email_trigger_node = EmailTriggerNode::new()?;
  registry.register_node(Arc::new(email_trigger_node))?;

  Ok(())
}
