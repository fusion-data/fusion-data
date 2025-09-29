//! # StartNode
//!
//! A node that serves as the entry point for a workflow.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, Node, NodeDefinition, NodeDefinitionBuilder, NodeExecutable,
  NodeExecutionContext, NodeExecutionError, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError,
  make_execution_data_map,
};

pub struct StartNodeV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinitionBuilder> for StartNodeV1 {
  type Error = RegistrationError;

  fn try_from(builder: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    let definition = builder.build()?;
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
  pub const NODE_KIND: &str = "hetumind_nodes::trigger::Start";

  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(StartNodeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

fn create_base() -> NodeDefinitionBuilder {
  let mut base = NodeDefinitionBuilder::default();
  base
    .kind(StartNode::NODE_KIND)
    .version(Version::new(1, 0, 0))
    .groups([NodeGroupKind::Trigger])
    .display_name("Start")
    .description("The entry point of the workflow.")
    .outputs(vec![]);
  base
}
