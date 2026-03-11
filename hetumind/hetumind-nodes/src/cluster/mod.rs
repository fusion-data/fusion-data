use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod ai_agent;

pub fn register_nodes(_node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  // Register AI nodes
  Ok(())
}
