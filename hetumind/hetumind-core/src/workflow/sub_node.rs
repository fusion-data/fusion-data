use std::sync::Arc;

use ahash::HashMap;
use async_trait::async_trait;

use crate::workflow::{ExecutionDataMap, ExecutionId, NodeDefinition, NodeExecutionError, Workflow};

pub struct NodeSupplyContext {
  pub execution_id: ExecutionId,
  pub workflow: Arc<Workflow>,

  pub input_data: ExecutionDataMap,

  pub request: serde_json::Value,
}

pub type SupplyResult = HashMap<String, serde_json::Value>;

/// Ability provider nodes do not participate in the data flow of workflow nodes.
/// This Sub node is directly called by the Root node
#[async_trait]
pub trait NodeSupplable {
  async fn init(&mut self, _context: NodeSupplyContext) -> Result<(), NodeExecutionError> {
    Ok(())
  }

  async fn supply(&self, _context: NodeSupplyContext) -> Result<SupplyResult, NodeExecutionError>;

  fn definition(&self) -> Arc<NodeDefinition>;
}

pub type NodeSupplier = Arc<dyn NodeSupplable + Send + Sync>;
