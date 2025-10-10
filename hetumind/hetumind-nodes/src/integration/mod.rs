use std::sync::Arc;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

mod http_request;

pub use http_request::HttpRequest;

pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  node_registry.register_node(Arc::new(HttpRequest::new()?))?;
  Ok(())
}
