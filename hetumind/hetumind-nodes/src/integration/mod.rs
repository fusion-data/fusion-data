use std::sync::Arc;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod http_request_node;

pub use http_request_node::HttpRequest;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  node_registry.register_node(Arc::new(HttpRequest::new()?))?;
  Ok(())
}
