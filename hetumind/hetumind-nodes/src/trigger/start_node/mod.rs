//! # StartNode
//!
//! A node that serves as the entry point for a workflow.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, NodeDefinition, NodeExecutionContext, NodeExecutionError,
  NodeExecutor, NodeGroupKind, NodeKind, make_execution_data_map,
};

pub struct StartNode {
  definition: Arc<NodeDefinition>,
}

impl Default for StartNode {
  fn default() -> Self {
    Self { definition: Arc::new(create_definition()) }
  }
}

#[async_trait]
impl NodeExecutor for StartNode {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, _context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]))
  }
}

impl StartNode {
  pub const NODE_KIND: &str = "hetumind_nodes::trigger::Start";
}

fn create_definition() -> NodeDefinition {
  NodeDefinition::builder()
    .kind(NodeKind::from(StartNode::NODE_KIND))
    .versions(vec![1])
    .groups(vec![NodeGroupKind::Trigger])
    .display_name("Start")
    .description("The entry point of the workflow.")
    .outputs(vec![])
    .build()
}
