//! # StartNode
//!
//! A node that serves as the entry point for a workflow.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ExecutionDataItems, ExecutionDataMap, FlowNode, FlowNodeRef, Node, NodeConnectionKind, NodeDescription,
  NodeExecutionContext, NodeExecutionError, NodeGroupKind, NodeType, RegistrationError, make_execution_data_map,
};

use crate::constants::START_TRIGGER_NODE_KIND;

pub struct StartNodeV1 {
  definition: Arc<NodeDescription>,
}

impl TryFrom<NodeDescription> for StartNodeV1 {
  type Error = RegistrationError;

  fn try_from(definition: NodeDescription) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl FlowNode for StartNodeV1 {
  fn description(&self) -> Arc<NodeDescription> {
    self.definition.clone()
  }

  async fn execute(&self, _context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    Ok(make_execution_data_map(vec![(NodeConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]))
  }
}

pub struct StartNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl Node for StartNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[FlowNodeRef] {
    &self.executors
  }

  fn node_type(&self) -> NodeType {
    self.executors[0].description().node_type.clone()
  }
}

impl StartNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_base();
    let executors: Vec<FlowNodeRef> = vec![Arc::new(StartNodeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.description().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

fn create_base() -> NodeDescription {
  NodeDescription::new(START_TRIGGER_NODE_KIND, "Start")
    .add_group(NodeGroupKind::Trigger)
    .with_description("The entry point of the workflow.")
}
