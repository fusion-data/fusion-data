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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::{AgentMetrics, TaskStatusInfo};

  #[test]
  fn test_heartbeat_request_creation() {
    let agent_id = Uuid::new_v4();
    let metrics = AgentMetrics::default();
    let running_tasks = vec![TaskStatusInfo {
      task_id: Uuid::now_v7(),
      status: crate::types::TaskInstanceStatus::Running,
      progress: Some(0.5),
      agent_id: agent_id.clone(),
      start_time: Some(1234567890),
    }];

    let heartbeat = HeartbeatRequest {
      agent_id,
      timestamp: 1234567890,
      status: AgentStatus::Online,
      running_tasks: running_tasks.clone(),
      metrics: metrics.clone(),
      last_task_id: Some("task-456".to_string()),
    };

    assert_eq!(heartbeat.agent_id, agent_id);
    assert_eq!(heartbeat.status, AgentStatus::Online);
    assert_eq!(heartbeat.running_tasks.len(), 1);
    assert_eq!(heartbeat.last_task_id, Some("task-456".to_string()));
  }

  #[test]
  fn test_heartbeat_request_serialization() {
    let agent_id = Uuid::new_v4();
    let metrics = AgentMetrics::default();
    let running_tasks = vec![];

    let heartbeat = HeartbeatRequest {
      agent_id,
      timestamp: 1234567890,
      status: AgentStatus::Offline,
      running_tasks,
      metrics,
      last_task_id: None,
    };

    let serialized = serde_json::to_string(&heartbeat).unwrap();
    let deserialized: HeartbeatRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.agent_id, agent_id);
    assert_eq!(deserialized.status, AgentStatus::Offline);
    assert!(deserialized.running_tasks.is_empty());
  }
}
