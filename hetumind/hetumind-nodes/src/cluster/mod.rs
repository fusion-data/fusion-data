use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod ai_agent;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  // Register AI nodes
  ai_agent::register_nodes(node_registry)?;
  Ok(())
}
