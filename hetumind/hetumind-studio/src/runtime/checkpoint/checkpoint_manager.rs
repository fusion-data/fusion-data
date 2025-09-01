use std::sync::Arc;

use crate::runtime::execution::ExecutionStore;
use hetumind_core::workflow::ExecutionId;
use log::info;
use fusion_common::time::now;

use super::{CheckpointConfig, CheckpointError, ExecutionCheckpoint, ExecutionState};

pub struct CheckpointManager {
  /// 状态存储
  state_store: Arc<dyn ExecutionStore>,
  /// 检查点配置
  config: CheckpointConfig,
}

impl CheckpointManager {
  pub async fn create_checkpoint(
    &self,
    execution_id: &ExecutionId,
    state: ExecutionState,
  ) -> Result<(), CheckpointError> {
    let checkpoint = ExecutionCheckpoint {
      execution_id: execution_id.clone(),
      timestamp: now(),
      execution_state: state,
      completed_nodes: todo!(),
      current_nodes: todo!(),
      pending_tasks: todo!(),
      intermediate_data: todo!(),
    };

    self.state_store.save_checkpoint(checkpoint).await?;

    info!("创建执行检查点");

    Ok(())
  }

  pub async fn restore_from_checkpoint(
    &self,
    execution_id: ExecutionId,
  ) -> Result<Option<ExecutionState>, CheckpointError> {
    if let Some(checkpoint) = self.state_store.load_latest_checkpoint(&execution_id).await? {
      info!("从检查点恢复执行状态");

      Ok(Some(checkpoint.execution_state))
    } else {
      Ok(None)
    }
  }
}
