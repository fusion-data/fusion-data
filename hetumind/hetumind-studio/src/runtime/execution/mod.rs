use async_trait::async_trait;
use hetumind_core::workflow::{Execution, ExecutionId, ExecutionStatus, WorkflowExecutionError};

use crate::runtime::checkpoint::{CheckpointError, ExecutionCheckpoint};

#[async_trait]
pub trait ExecutionStore: Send + Sync {
  /// 保存执行
  async fn save_execution(&self, execution: &Execution) -> Result<(), WorkflowExecutionError>;

  /// 根据执行ID获取执行
  async fn get_execution(&self, id: &ExecutionId) -> Result<Option<Execution>, WorkflowExecutionError>;

  async fn get_execution_status(&self, id: &ExecutionId) -> Result<ExecutionStatus, WorkflowExecutionError>;

  async fn update_execution_status(
    &self,
    id: &ExecutionId,
    status: ExecutionStatus,
  ) -> Result<(), WorkflowExecutionError>;

  async fn save_checkpoint(&self, checkpoint: ExecutionCheckpoint) -> Result<(), CheckpointError>;

  async fn load_latest_checkpoint(
    &self,
    execution_id: &ExecutionId,
  ) -> Result<Option<ExecutionCheckpoint>, CheckpointError>;
}
