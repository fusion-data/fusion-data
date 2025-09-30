use fusion_common::ahash::HashMap;
use fusion_common::time::OffsetDateTime;
#[cfg(feature = "with-db")]
use modelsql::generate_enum_i32_to_sea_query_value;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

mod error;
mod task_queue;
mod task_worker;

pub use error::{QueueError, WorkerError};
pub use task_queue::TaskQueue;
pub use task_worker::TaskWorker;

use crate::workflow::{ExecutionId, WorkflowId};

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum TaskPriority {
  Low = 10,
  Normal = 20,
  High = 30,
  Critical = 40,
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum TaskStatus {
  Pending = 1,
  Processing = 10,
  Failed = 98,
  Cancelled = 99,
  Completed = 100,
}

#[cfg(feature = "with-db")]
generate_enum_i32_to_sea_query_value!(Enum: TaskPriority, Enum: TaskStatus,);

/// 队列任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueTask {
  pub id: String,
  pub task_type: String,
  pub execution_id: ExecutionId,
  pub workflow_id: WorkflowId,
  pub priority: TaskPriority,
  pub payload: serde_json::Value,
  pub retry_count: u32,
  pub max_retries: u32,
  pub created_at: OffsetDateTime,
  pub scheduled_at: Option<OffsetDateTime>,
  pub metadata: HashMap<String, String>,
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
  pub task_id: String,
  pub execution_id: ExecutionId,
  pub status: TaskStatus,
  pub result: Option<serde_json::Value>,
  pub error: Option<String>,
  pub completed_at: OffsetDateTime,
  pub duration_ms: u64,
}

/// 队列统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
  pub pending: u64,
  pub processing: u64,
  pub completed: u64,
  pub failed: u64,
  pub delayed: u64,
}
