mod concurrency_controller;
mod task_scheduler;

use std::sync::Arc;

pub use concurrency_controller::*;
pub use task_scheduler::*;

use ahash::{HashMap, HashSet};
use hetumind_core::{
  task::TaskPriority,
  workflow::{ExecutionData, ExecutionId, NodeExecutionError, NodeName, WorkflowNode},
};
use serde::{Deserialize, Serialize};
use fusion_common::time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTask {
  /// 执行ID
  pub execution_id: ExecutionId,
  /// 节点ID
  pub node_name: NodeName,
  /// 节点信息
  /// TODO 是否应该使用 Workflow ?
  pub node: Arc<WorkflowNode>,
  /// 输入数据
  pub input_data: Arc<Vec<ExecutionData>>,
  /// 优先级
  pub priority: TaskPriority,
  /// 创建时间
  pub created_at: OffsetDateTime,
  /// 超时时间
  pub timeout: Option<std::time::Duration>,
}

#[derive(Debug)]
pub struct WaitingTask {
  /// 任务信息
  pub task: ExecutionTask,
  /// 等待的依赖节点
  pub waiting_for: HashSet<NodeName>,
  /// 已收到的输入
  pub received_inputs: HashMap<NodeName, Vec<ExecutionData>>,
}

#[derive(Debug)]
pub struct RunningTask {
  /// 任务信息
  pub task: ExecutionTask,
  /// 执行句柄
  pub handle: tokio::task::JoinHandle<Result<Vec<ExecutionData>, NodeExecutionError>>,
  /// 开始时间
  pub started_at: OffsetDateTime,
}

#[derive(Debug)]
pub enum SchedulerCommand {
  ScheduleTask(ExecutionTask),
  TaskCompleted {
    node_name: NodeName,
    execution_id: ExecutionId,
    result: Result<Vec<ExecutionData>, NodeExecutionError>,
  },
  CancelTask(NodeName),
  PauseExecution(ExecutionId),
  ResumeExecution(ExecutionId),
  CancelExecution(ExecutionId),
  Shutdown,
}
