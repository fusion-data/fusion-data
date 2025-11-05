mod http_method;
mod http_request_v1;

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, FlowNodeRef, NodeKind, RegistrationError},
};

pub use http_method::*;
pub use http_request_v1::*;

pub struct HttpRequest {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl HttpRequest {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_definition()?;
    let executors: Vec<FlowNodeRef> = vec![Arc::new(HttpRequestV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for HttpRequest {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[FlowNodeRef] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}
