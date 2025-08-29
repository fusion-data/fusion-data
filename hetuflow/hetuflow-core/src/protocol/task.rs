use serde::{Deserialize, Serialize};
use ultimate_common::time::now_epoch_millis;
use uuid::Uuid;

use crate::{
  models::{TaskEntity, TaskInstanceEntity, TaskMetrics},
  types::{TaskControlKind, TaskInstanceStatus},
};

/// 任务分发请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
  pub task_instance: TaskInstanceEntity,
  pub task: TaskEntity,
}

impl ScheduledTask {
  /// 创建新的调度任务
  pub fn new(task_instance: TaskInstanceEntity, task: TaskEntity) -> Self {
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
  pub fn tags(&self) -> &[String] {
    self.task.tags.as_ref()
  }

  /// 检查任务是否匹配指定的标签
  pub fn matches_tags(&self, required_tags: &[String]) -> bool {
    if required_tags.is_empty() {
      return true;
    }

    required_tags.iter().all(|tag| self.task.tags.contains(tag))
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
pub struct TaskInstanceUpdated {
  /// 任务实例 ID
  // TODO: 是否应该添加 task_instance_id？（由客户端生成 task instance id，避免重复或遗漏）
  pub task_instance_id: Uuid,
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

impl TaskInstanceUpdated {
  pub fn new(agent_id: Uuid, task_id: Uuid, task_instance_id: Uuid, status: TaskInstanceStatus) -> Self {
    Self {
      task_instance_id,
      agent_id,
      task_id,
      status,
      timestamp: now_epoch_millis(),
      output: None,
      error_message: None,
      exit_code: None,
      metrics: None,
      progress: None,
    }
  }

  pub fn with_output(mut self, output: impl Into<String>) -> Self {
    self.output = Some(output.into());
    self
  }

  pub fn with_error_message(mut self, error_message: impl Into<String>) -> Self {
    self.error_message = Some(error_message.into());
    self
  }

  pub fn with_exit_code(mut self, exit_code: i32) -> Self {
    self.exit_code = Some(exit_code);
    self
  }

  pub fn with_metrics(mut self, metrics: TaskMetrics) -> Self {
    self.metrics = Some(metrics);
    self
  }

  pub fn with_progress(mut self, progress: f64) -> Self {
    self.progress = Some(progress);
    self
  }
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
  pub tasks: Vec<ScheduledTask>, // 可执行任务列表
  pub has_more: bool,            // 是否还有更多任务
  pub next_poll_interval: u32,   // 下次拉取间隔(秒)
}

/// 创建任务实例请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskInstanceRequest {
  pub task_id: Uuid,          // 任务ID
  pub agent_id: Uuid,         // Agent ID
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

/// 任务执行错误
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskExecutionError {
  pub task_id: Uuid,                      // 任务ID
  pub instance_id: Option<Uuid>,          // 任务实例ID
  pub error_type: TaskExecutionErrorType, // 错误类型
  pub message: String,                    // 错误消息
  pub retry_count: i32,                   // 当前重试次数
  pub max_retries: i32,                   // 最大重试次数
  pub timestamp: i64,                     // 错误时间戳
}

/// 任务执行错误类型
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TaskExecutionErrorType {
  /// 任务被取消
  Cancelled,
  /// 进程启动失败
  ProcessStartFailed,
  /// 进程执行超时
  ProcessTimeout,
  /// 进程被杀死
  ProcessKilled,
  /// 资源不足
  ResourceExhausted,
  /// 依赖检查失败
  DependencyCheckFailed,
  /// 配置错误
  ConfigurationError,
  /// 网络错误
  NetworkError,
}
