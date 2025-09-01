mod checkpoint_manager;
mod error;
mod state;

pub use checkpoint_manager::CheckpointManager;
pub use error::CheckpointError;
pub use state::{ExecutionState, NodeExecutionState};

use ahash::{HashMap, HashSet};
use hetumind_core::workflow::{ExecutionData, ExecutionId, NodeName};
use serde::{Deserialize, Serialize};
use fusion_common::time::OffsetDateTime;

use crate::runtime::task::ExecutionTask;

#[derive(Debug, Clone)]
pub struct CheckpointConfig {
  /// 检查点间隔（秒）
  pub interval_seconds: u64,
  /// 保留检查点数量
  pub max_checkpoints: u32,
  /// 是否启用增量检查点
  pub incremental: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
  /// 执行ID
  pub execution_id: ExecutionId,
  /// 检查点时间
  pub timestamp: OffsetDateTime,
  /// 执行状态
  pub execution_state: ExecutionState,
  /// 已完成的节点
  pub completed_nodes: HashSet<NodeName>,
  /// 当前执行的节点
  pub current_nodes: HashMap<NodeName, NodeExecutionState>,
  /// 待执行的任务队列
  pub pending_tasks: Vec<ExecutionTask>,
  /// 中间数据
  pub intermediate_data: HashMap<NodeName, Vec<ExecutionData>>,
}
