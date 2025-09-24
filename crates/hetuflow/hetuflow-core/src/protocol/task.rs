use fusion_common::time::OffsetDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  models::{SchedTask, SchedTaskInstance, TaskMetrics},
  types::{Labels, TaskInstanceStatus},
};

/// 任务分发请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
  pub task_instance: SchedTaskInstance,
  pub task: SchedTask,
}

impl ScheduledTask {
  /// 创建新的调度任务
  pub fn new(task_instance: SchedTaskInstance, task: SchedTask) -> Self {
    Self { task_instance, task }
  }

  pub fn task_instance_id(&self) -> Uuid {
    self.task_instance.id
  }

  /// 获取任务ID
  pub fn task_id(&self) -> Uuid {
    self.task.id
  }

  /// 获取Job ID
  pub fn job_id(&self) -> Uuid {
    self.task.job_id
  }

  /// 获取调度ID
  pub fn schedule_id(&self) -> Option<Uuid> {
    self.task.schedule_id
  }

  /// 获取任务优先级
  pub fn priority(&self) -> i32 {
    self.task.priority
  }

  /// 获取任务标签
  pub fn labels(&self) -> Labels {
    self.task.config.labels()
  }

  /// 检查任务是否匹配指定的标签
  pub fn match_label(&self, label: &str, value: &str) -> bool {
    match self.task.config.labels.as_ref() {
      Some(labels) => labels.get(label).is_some_and(|v| v == value),
      None => false,
    }
  }

  /// 检查任务是否为定时任务
  pub fn is_scheduled(&self) -> bool {
    self.task.schedule_id.is_some()
  }

  /// 检查任务是否为手动触发任务
  pub fn is_manual(&self) -> bool {
    self.task.schedule_id.is_none()
  }

  /// 获取调度类型描述
  pub fn schedule_type_description(&self) -> &str {
    self.task.schedule_kind.as_ref()
  }
}

/// 任务状态更新 (Reports status for a 'Task', which the server records as a 'TaskInstance')
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstanceChanged {
  /// 任务实例 ID
  pub instance_id: Uuid,
  /// Agent ID
  pub agent_id: String,
  /// 执行状态
  pub status: TaskInstanceStatus,
  /// 状态更新时间
  pub epoch_millis: i64,
  /// 任务数据
  pub data: Option<String>,
  /// 错误信息
  pub error_message: Option<String>,
  /// 执行指标
  pub metrics: Option<TaskMetrics>,
}

impl TaskInstanceChanged {
  pub fn with_status(&mut self, status: TaskInstanceStatus) -> &Self {
    self.status = status;
    self
  }

  pub fn with_output(&mut self, output: impl Into<String>) -> &Self {
    self.data = Some(output.into());
    self
  }

  pub fn with_error_message(&mut self, error_message: impl Into<String>) -> &Self {
    self.error_message = Some(error_message.into());
    self
  }

  pub fn with_metrics(&mut self, metrics: TaskMetrics) -> &Self {
    self.metrics = Some(metrics);
    self
  }
}

/// Task pull request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AcquireTaskRequest {
  /// Agent ID
  pub agent_id: String,
  /// The maximum number of concurrent tasks allowed by the agent
  pub max_tasks: u32,
  /// The labels currently owned by the Agent are used to filter tasks
  pub labels: Labels,
  /// Poll task count
  pub acquire_count: u32,
  /// The maximum execution time, only tasks that are lte this time will be retrieved
  pub max_scheduled_at: OffsetDateTime,
}

/// Task response, for task pull requests or direct task assignments from Server to Agent
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AcquireTaskResponse {
  pub tasks: Vec<ScheduledTask>, // 可执行任务列表
  pub has_more: bool,            // 是否还有更多任务
  pub next_poll_interval: u32,   // 下次拉取间隔(秒)
}

/// 创建任务实例请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskInstanceRequest {
  pub task_id: Uuid,          // 任务ID
  pub agent_id: String,       // Agent ID
  pub retry_count: i32,       // 重试次数
  pub reason: Option<String>, // 创建原因
}

/// 创建任务实例响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskInstanceResponse {
  pub success: bool,             // 是否成功
  pub instance_id: Option<Uuid>, // 任务实例ID
  pub message: String,           // 响应消息
}

/// 退避策略
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BackoffStrategy {
  /// 固定间隔
  Fixed,
  /// 线性退避
  Linear,
  /// 指数退避
  Exponential,
  /// 带抖动的指数退避
  ExponentialWithJitter,
}

impl Default for BackoffStrategy {
  fn default() -> Self {
    Self::ExponentialWithJitter
  }
}

/// 任务执行结果
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskExecutionResult {
  pub task_id: Uuid,                 // 任务ID
  pub instance_id: Uuid,             // 任务实例ID
  pub success: bool,                 // 是否成功
  pub exit_code: Option<i32>,        // 退出码
  pub output: Option<String>,        // 输出内容
  pub error_message: Option<String>, // 错误信息
  pub metrics: Option<TaskMetrics>,  // 执行指标
  pub duration_ms: u64,              // 执行时长(毫秒)
}
