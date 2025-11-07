use std::sync::Arc;

use async_trait::async_trait;

use crate::workflow::{ExecutionDataMap, NodeDescription, NodeExecutionContext, NodeExecutionError};

/// Can be called by workflow
#[async_trait]
pub trait FlowNode {
  /// Initialize the node. This can be used to implement node initialization logic, such as loading configuration, initializing resources, etc.
  async fn init(&mut self, _context: &NodeExecutionContext) -> Result<(), NodeExecutionError> {
    Ok(())
  }

  /// Execute the node
  ///
  /// Returns:
  /// - On success, returns data for multiple output ports, with the first output port starting from 0
  /// - On failure, returns an error
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError>;

  /// Get Node definition
  fn description(&self) -> Arc<NodeDescription>;
}

/// Node Executor Type
pub type FlowNodeRef = Arc<dyn FlowNode + Send + Sync>;
