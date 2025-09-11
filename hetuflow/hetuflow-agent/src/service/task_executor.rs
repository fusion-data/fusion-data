use std::{
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};

use fusion_common::{ahash::HashMap, time::now_epoch_millis};
use fusion_core::DataError;
use log::{debug, error, info, warn};
use tokio::sync::RwLock;
use uuid::Uuid;

use hetuflow_core::{
  protocol::{ProcessInfo, ScheduledTask, TaskExecutionError, TaskInstanceUpdated, WebSocketEvent},
  types::{EventKind, TaskInstanceStatus},
};

use crate::{
  service::{ConnectionManager, ProcessManager},
  setting::HetuflowAgentSetting,
};

/// 任务执行状态
#[derive(Debug, Clone)]
pub struct TaskExecution {
  /// 任务实例ID
  pub instance_id: Uuid,
  /// 调度任务
  pub task: Arc<ScheduledTask>,
  /// 进程执行 ID
  pub process_id: Option<Uuid>,
  /// 进程信息
  pub process_info: Option<ProcessInfo>,
  /// 开始时间
  pub started_at: i64,
  /// 状态
  pub status: TaskInstanceStatus,
  /// 取消标志
  pub cancelled: Arc<AtomicBool>,
}

/// 任务执行器。负责执行具体的任务，包括进程管理和状态上报
pub struct TaskExecutor {
  /// Agent ID
  setting: Arc<HetuflowAgentSetting>,
  /// 进程管理器
  process_manager: Arc<ProcessManager>,
  /// 活跃任务执行
  active_executions: Arc<RwLock<HashMap<Uuid, TaskExecution>>>,
  /// 连接管理器
  connection_manager: Arc<ConnectionManager>,
  scheduled_task_rx: Arc<kanal::AsyncReceiver<ScheduledTask>>,
}

impl TaskExecutor {
  /// 创建新的任务执行器
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    process_manager: Arc<ProcessManager>,
    connection_manager: Arc<ConnectionManager>,
    scheduled_task_rx: kanal::AsyncReceiver<ScheduledTask>,
  ) -> Self {
    Self {
      setting,
      process_manager,
      active_executions: Arc::new(RwLock::new(HashMap::default())),
      connection_manager,
      scheduled_task_rx: Arc::new(scheduled_task_rx),
    }
  }

  /// 启动任务执行器
  pub async fn run_loop(&self) -> Result<(), DataError> {
    info!("Starting TaskExecutor for agent {}", self.setting.agent_id);

    loop {
      match self.scheduled_task_rx.recv().await {
        Ok(task) => self.execute_task(task).await,
        Err(e) => {
          info!("The scheduled_task_rx channel closed: {}", e);
          break;
        }
      }
    }

    self.stop().await
  }

  /// 停止任务执行器
  async fn stop(&self) -> Result<(), DataError> {
    info!("Stopping TaskExecutor for agent {}", self.setting.agent_id);

    // 取消所有活跃任务
    let executions = self.active_executions.read().await;
    for execution in executions.values() {
      execution.cancelled.store(true, Ordering::SeqCst);
      if let Some(process_id) = execution.process_id.as_ref()
        && let Err(e) = self.process_manager.kill_process(process_id).await
      {
        warn!("Failed to kill process {}: {}", process_id, e);
      }
    }
    drop(executions);

    // 等待所有任务完成
    let mut attempts = 0;
    while !self.active_executions.read().await.is_empty() && attempts < 30 {
      tokio::time::sleep(Duration::from_millis(100)).await;
      attempts += 1;
    }

    Ok(())
  }

  /// 取消任务
  pub async fn cancel_task(&self, instance_id: Uuid) -> Result<(), DataError> {
    let mut executions = self.active_executions.write().await;
    if let Some(execution) = executions.remove(&instance_id) {
      execution.cancelled.store(true, Ordering::SeqCst);
      if let Some(process_id) = execution.process_id.as_ref() {
        self.process_manager.kill_process(process_id).await?;
      }
      let event = WebSocketEvent::new(
        EventKind::TaskChangedEvent,
        TaskInstanceUpdated {
          instance_id,
          agent_id: self.setting.agent_id,
          timestamp: now_epoch_millis(),
          output: None,
          error_message: None,
          exit_code: None,
          metrics: None,
          status: TaskInstanceStatus::Cancelled,
        },
      );
      self.connection_manager.send_event(event)?;
    }
    Ok(())
  }

  /// 获取活跃任务列表
  pub async fn get_active_tasks(&self) -> Vec<TaskExecution> {
    self.active_executions.read().await.values().cloned().collect()
  }

  /// 执行任务
  pub async fn execute_task(&self, task: ScheduledTask) {
    let instance_id = task.task_instance_id();
    let execution = TaskExecution {
      instance_id,
      task: Arc::new(task),
      process_id: None,
      process_info: None,
      started_at: now_epoch_millis(),
      status: TaskInstanceStatus::Running,
      cancelled: Arc::new(AtomicBool::new(false)),
    };

    // 执行任务（单次执行）
    let result = self.execute_single_attempt(execution).await;
    match result {
      Ok(execution) => {
        if execution.process_id.is_some() {
          self.active_executions.write().await.insert(instance_id, execution);
        } else {
          error!("Task {} executed without process ID", instance_id);
        }
      }
      Err(error) => {
        let event = self.process_execution_error(instance_id, error);
        if let Err(e) = self.connection_manager.send_event(event) {
          error!("Failed to send event: {:?}", e);
        }
      }
    }
  }

  /// 执行单次任务尝试
  async fn execute_single_attempt(&self, mut execution: TaskExecution) -> Result<TaskExecution, TaskExecutionError> {
    // 检查是否被取消
    if execution.cancelled.load(Ordering::SeqCst) {
      return Err(TaskExecutionError::Cancelled);
    }

    // 获取任务配置
    let task_config = execution.task.task.config.as_ref().ok_or(TaskExecutionError::ConfigurationError)?;

    // 准备环境变量
    let environment =
      if let Some(value) = execution.task.task.environment.clone() { serde_json::from_value(value).ok() } else { None };

    execution.started_at = now_epoch_millis();

    // 使用ProcessManager启动进程
    let process_id = self
      .process_manager
      .spawn_process(
        execution.instance_id,
        &task_config.cmd,
        &task_config.args,
        task_config.working_directory.as_deref(),
        environment.as_ref(),
        task_config.resource_limits.as_ref(),
      )
      .await
      .map_err(|_e| TaskExecutionError::ProcessStartFailed)?;

    execution.process_id = Some(process_id);
    debug!("Started process {} for task instance {}", process_id, execution.task.task_instance.id);

    Ok(execution)
  }

  fn process_execution_error(&self, instance_id: Uuid, error: TaskExecutionError) -> WebSocketEvent {
    let mut payload = TaskInstanceUpdated {
      instance_id,
      agent_id: self.setting.agent_id,
      timestamp: now_epoch_millis(),
      output: None,
      error_message: None,
      exit_code: None,
      metrics: None,
      status: TaskInstanceStatus::Failed,
    };
    match error {
      TaskExecutionError::Cancelled => payload.with_status(TaskInstanceStatus::Cancelled),
      TaskExecutionError::ProcessStartFailed => payload.with_error_message("Process start failed"),
      TaskExecutionError::ProcessTimeout => payload.with_status(TaskInstanceStatus::Timeout),
      TaskExecutionError::ProcessKilled => payload.with_error_message("Killed"),
      TaskExecutionError::ResourceExhausted => payload.with_error_message("Resource exhausted"),
      TaskExecutionError::DependencyCheckFailed => payload.with_error_message("Dependency check failed"),
      TaskExecutionError::ConfigurationError => payload.with_error_message("Configuration error"),
      TaskExecutionError::NetworkError => payload.with_error_message("Network error"),
      TaskExecutionError::Failed => payload.with_error_message("Failed"),
    };
    WebSocketEvent::new(EventKind::TaskChangedEvent, payload)
  }
}
