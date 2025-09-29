mod http_method;
mod http_request_v1;

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeExecutor, NodeKind, RegistrationError},
};

pub use http_method::*;
pub use http_request_v1::*;

pub struct HttpRequest {
  default_version: Version,
  versions: Vec<Version>,
  executors: Vec<NodeExecutor>,
}

impl HttpRequest {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_definition()?;
    let executors: Vec<NodeExecutor> = vec![Arc::new(HttpRequestV1::try_from(base)?)];
    let versions: Vec<Version> = executors.iter().map(|node| node.definition().version.clone()).collect();
    let default_version = versions.iter().max().unwrap().clone();
    Ok(Self { default_version, versions, executors })
  }
}

impl Node for HttpRequest {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[NodeExecutor] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}
