use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod http_request_node;

pub use http_request_node::HttpRequestNode;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  node_registry.register_node(HttpRequestNode::default())?;
  Ok(())
}
