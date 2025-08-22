use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  models::{JobConfig, TaskMetrics},
  types::{ScheduleKind, TaskControlKind, TaskInstanceStatus},
};

/// 任务分发请求 (Represents a 'Task')
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DispatchTaskPayload {
  /// 归属的 JobEntity ID
  pub job_id: Uuid,
  /// 本次 Task 的唯一标识 (UUID)
  pub task_id: Uuid,
  /// 任务名称
  pub task_name: Option<String>,
  /// 任务类型
  pub schedule_kind: ScheduleKind,
  /// 执行命令
  pub command: String,
  /// Cron 表达式 (仅 CRON 类型)
  pub cron_expression: Option<String>,
  /// 环境变量
  pub environment: HashMap<String, String>,
  /// 任务配置
  pub config: JobConfig,
  /// 调度时间戳
  pub scheduled_at: i64,
  /// 任务优先级
  pub priority: u8,
  /// 依赖任务ID列表
  pub dependencies: Vec<Uuid>,
}

/// 任务分发响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DispatchTaskResponse {
  /// 是否成功接收任务
  pub success: bool,
  /// 响应消息
  pub message: String,
  /// 任务ID
  pub task_id: Uuid,
}

/// 任务状态更新 (Reports status for a 'Task', which the server records as a 'TaskInstance')
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskInstanceUpdated {
  /// 正在上报状态的 Task ID
  pub task_id: Uuid,
  /// Agent ID
  pub agent_id: Uuid,
  /// 执行状态
  pub status: TaskInstanceStatus,
  /// 状态更新时间
  pub timestamp: i64,
  /// 任务输出
  pub output: Option<String>,
  /// 错误信息
  pub error_message: Option<String>,
  /// 退出码
  pub exit_code: Option<i32>,
  /// 执行指标
  pub metrics: Option<TaskMetrics>,
  /// 执行进度 (0.0-1.0)
  pub progress: Option<f64>,
}

/// 任务控制指令
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskControl {
  pub task_id: Uuid,                 // 任务ID
  pub control_type: TaskControlKind, // 控制类型
  pub reason: Option<String>,        // 控制原因
  pub force: bool,                   // 是否强制执行
}

/// 任务拉取请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskPollRequest {
  pub agent_id: Uuid,          // Agent ID
  pub max_tasks: u32,          // 最大拉取任务数
  pub tags: Vec<String>,       // 当前 Agent 拥有的标签，用于过滤任务
  pub available_capacity: u32, // 可用容量
}

/// 任务拉取响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskPollResponse {
  pub tasks: Vec<DispatchTaskPayload>, // 可执行任务列表
  pub has_more: bool,                  // 是否还有更多任务
  pub next_poll_interval: u32,         // 下次拉取间隔(秒)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::JobConfig;

  #[test]
  fn test_dispatch_task_request_creation() {
    let job_id = Uuid::now_v7();
    let task_id = Uuid::now_v7();
    let config = JobConfig::default();

    let request = DispatchTaskPayload {
      job_id,
      task_id,
      task_name: Some("test-task".to_string()),
      schedule_kind: ScheduleKind::Cron,
      command: "echo hello".to_string(),
      cron_expression: Some("0 12 * * *".to_string()),
      environment: HashMap::from([("ENV".to_string(), "test".to_string())]),
      config: config.clone(),
      scheduled_at: 1234567890,
      priority: 5,
      dependencies: vec![Uuid::new_v4()],
    };

    assert_eq!(request.job_id, job_id);
    assert_eq!(request.task_id, task_id);
    assert_eq!(request.schedule_kind, ScheduleKind::Cron);
    assert_eq!(request.command, "echo hello");
  }

  #[test]
  fn test_dispatch_task_request_serialization() {
    let job_id = Uuid::now_v7();
    let task_id = Uuid::now_v7();
    let config = JobConfig::default();

    let request = DispatchTaskPayload {
      job_id,
      task_id,
      task_name: None,
      schedule_kind: ScheduleKind::Interval,
      command: "ls -la".to_string(),
      cron_expression: None,
      environment: HashMap::default(),
      config,
      scheduled_at: 1234567890,
      priority: 1,
      dependencies: vec![],
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: DispatchTaskPayload = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.job_id, job_id);
    assert_eq!(deserialized.schedule_kind, ScheduleKind::Interval);
  }

  #[test]
  fn test_dispatch_task_response_creation() {
    let task_id = Uuid::now_v7();
    let response = DispatchTaskResponse { success: true, message: "Task accepted".to_string(), task_id };

    assert!(response.success);
    assert_eq!(response.message, "Task accepted");
    assert_eq!(response.task_id, task_id);
  }

  #[test]
  fn test_dispatch_task_response_serialization() {
    let task_id = Uuid::now_v7();
    let response = DispatchTaskResponse { success: false, message: "Task rejected".to_string(), task_id };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: DispatchTaskResponse = serde_json::from_str(&serialized).unwrap();

    assert!(!deserialized.success);
    assert_eq!(deserialized.message, "Task rejected");
  }

  #[test]
  fn test_task_instance_update_creation() {
    let task_id = Uuid::now_v7();
    let agent_id = Uuid::new_v4();
    let metrics = TaskMetrics::default();

    let update = TaskInstanceUpdated {
      task_id,
      agent_id,
      status: TaskInstanceStatus::Running,
      timestamp: 1234567890,
      output: Some("output data".to_string()),
      error_message: None,
      exit_code: None,
      metrics: Some(metrics.clone()),
      progress: Some(0.75),
    };

    assert_eq!(update.task_id, task_id);
    assert_eq!(update.agent_id, agent_id);
    assert_eq!(update.status, TaskInstanceStatus::Running);
    assert_eq!(update.progress, Some(0.75));
  }

  #[test]
  fn test_task_instance_update_serialization() {
    let task_id = Uuid::now_v7();
    let agent_id = Uuid::new_v4();

    let update = TaskInstanceUpdated {
      task_id,
      agent_id,
      status: TaskInstanceStatus::Succeeded,
      timestamp: 1234567890,
      output: None,
      error_message: Some("error occurred".to_string()),
      exit_code: Some(1),
      metrics: None,
      progress: None,
    };

    let serialized = serde_json::to_string(&update).unwrap();
    let deserialized: TaskInstanceUpdated = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.task_id, task_id);
    assert_eq!(deserialized.status, TaskInstanceStatus::Succeeded);
    assert_eq!(deserialized.error_message, Some("error occurred".to_string()));
  }

  #[test]
  fn test_task_control_creation() {
    let task_id = Uuid::now_v7();

    let control = TaskControl {
      task_id,
      control_type: TaskControlKind::Stop,
      reason: Some("User requested".to_string()),
      force: true,
    };

    assert_eq!(control.task_id, task_id);
    assert_eq!(control.control_type, TaskControlKind::Stop);
    assert_eq!(control.reason, Some("User requested".to_string()));
    assert!(control.force);
  }

  #[test]
  fn test_task_control_serialization() {
    let task_id = Uuid::now_v7();

    let control = TaskControl { task_id, control_type: TaskControlKind::Pause, reason: None, force: false };

    let serialized = serde_json::to_string(&control).unwrap();
    let deserialized: TaskControl = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.task_id, task_id);
    assert_eq!(deserialized.control_type, TaskControlKind::Pause);
    assert_eq!(deserialized.reason, None);
    assert!(!deserialized.force);
  }

  #[test]
  fn test_task_poll_request_creation() {
    let agent_id = Uuid::new_v4();

    let request = TaskPollRequest {
      agent_id,
      max_tasks: 10,
      tags: vec!["bash".to_string(), "python".to_string()],
      available_capacity: 5,
    };

    assert_eq!(request.agent_id, agent_id);
    assert_eq!(request.max_tasks, 10);
    assert_eq!(request.tags, vec!["bash", "python"]);
    assert_eq!(request.available_capacity, 5);
  }

  #[test]
  fn test_task_poll_request_serialization() {
    let agent_id = Uuid::new_v4();

    let request = TaskPollRequest { agent_id, max_tasks: 1, tags: vec![], available_capacity: 0 };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: TaskPollRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.agent_id, agent_id);
    assert_eq!(deserialized.max_tasks, 1);
  }

  #[test]
  fn test_task_poll_response_creation() {
    let tasks = vec![];

    let response = TaskPollResponse { tasks: tasks.clone(), has_more: false, next_poll_interval: 30 };

    assert!(response.tasks.is_empty());
    assert!(!response.has_more);
    assert_eq!(response.next_poll_interval, 30);
  }

  #[test]
  fn test_task_poll_response_serialization() {
    let tasks = vec![];

    let response = TaskPollResponse { tasks, has_more: true, next_poll_interval: 60 };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: TaskPollResponse = serde_json::from_str(&serialized).unwrap();

    assert!(deserialized.has_more);
    assert_eq!(deserialized.next_poll_interval, 60);
  }
}
