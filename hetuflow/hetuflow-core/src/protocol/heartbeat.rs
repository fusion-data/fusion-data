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

/// 心跳响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeartbeatResponse {
  /// 响应状态
  pub success: bool,
  /// 服务器时间戳
  pub server_timestamp: i64,
  /// 响应消息
  pub message: Option<String>,
  /// 服务器配置更新（可选）
  pub config_updates: Option<serde_json::Value>,
}
