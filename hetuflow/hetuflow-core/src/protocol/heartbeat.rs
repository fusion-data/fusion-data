use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  models::{AgentMetrics, TaskStatusInfo},
  types::AgentStatus,
};

/// 心跳请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeartbeatRequest {
  /// Agent ID
  pub agent_id: Uuid,
  /// 心跳时间
  pub timestamp: i64,
  /// Agent 状态
  pub status: AgentStatus,
  /// 运行中的任务状态信息
  pub running_tasks: Vec<TaskStatusInfo>,
  /// Agent 性能指标
  pub metrics: AgentMetrics,
  /// 最后处理的任务ID
  pub last_task_id: Option<String>,
}
