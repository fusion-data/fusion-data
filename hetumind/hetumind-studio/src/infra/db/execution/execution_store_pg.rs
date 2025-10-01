use async_trait::async_trait;
use hetumind_core::workflow::{Execution, ExecutionForUpdate, ExecutionId, ExecutionStatus, WorkflowExecutionError};
use fusionsql::ModelManager;

use crate::{
  domain::workflow::ExecutionBmc,
  runtime::{
    checkpoint::{CheckpointError, ExecutionCheckpoint},
    execution::ExecutionStore,
  },
};

pub struct ExecutionStorePg {
  mm: ModelManager,
}

impl ExecutionStorePg {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }
}

#[async_trait]
impl ExecutionStore for ExecutionStorePg {
  async fn save_execution(&self, execution: &Execution) -> Result<(), WorkflowExecutionError> {
    let execution_for_update = ExecutionForUpdate::from(execution.clone());
    ExecutionBmc::update_by_id(&self.mm, execution.id.clone(), execution_for_update).await?;
    Ok(())
  }

  async fn get_execution(&self, id: &ExecutionId) -> Result<Option<Execution>, WorkflowExecutionError> {
    todo!()
  }

  async fn get_execution_status(&self, id: &ExecutionId) -> Result<ExecutionStatus, WorkflowExecutionError> {
    todo!()
  }

  async fn update_execution_status(
    &self,
    id: &ExecutionId,
    status: ExecutionStatus,
  ) -> Result<(), WorkflowExecutionError> {
    todo!()
  }

  async fn save_checkpoint(&self, checkpoint: ExecutionCheckpoint) -> Result<(), CheckpointError> {
    todo!()
  }

  async fn load_latest_checkpoint(
    &self,
    execution_id: &ExecutionId,
  ) -> Result<Option<ExecutionCheckpoint>, CheckpointError> {
    todo!()
  }
}
