//! # StartNode
//!
//! A node that serves as the entry point for a workflow.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, Node, NodeDefinition, NodeExecutable, NodeExecutionContext,
  NodeExecutionError, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError, make_execution_data_map,
};

use crate::constants::START_TRIGGER_NODE_KIND;

pub struct StartNodeV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinition> for StartNodeV1 {
  type Error = RegistrationError;

  fn try_from(definition: NodeDefinition) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl NodeExecutable for StartNodeV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, _context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]))
  }
}

pub struct StartNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl Node for StartNode {
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

impl StartNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(StartNodeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

fn create_base() -> NodeDefinition {
  NodeDefinition::new(START_TRIGGER_NODE_KIND, "Start")
    .add_group(NodeGroupKind::Trigger)
    .with_description("The entry point of the workflow.")
}
