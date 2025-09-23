use std::sync::Arc;

use fusion_common::time::now_epoch_millis;
use hetuflow_core::{
  protocol::{ScheduledTask, TaskExecutionError, TaskInstanceChanged, WebSocketEvent},
  types::TaskInstanceStatus,
};
use log::{error, info};
use mea::mpsc;
use uuid::Uuid;

use crate::{connection::ConnectionManager, process::ProcessManager, setting::HetuflowAgentSetting};

pub struct TaskExecuteRunner {
  setting: Arc<HetuflowAgentSetting>,
  connection_manager: Arc<ConnectionManager>,
  process_manager: Arc<ProcessManager>,
  scheduled_task_rx: mpsc::UnboundedReceiver<ScheduledTask>,
}

impl TaskExecuteRunner {
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    connection_manager: Arc<ConnectionManager>,
    process_manager: Arc<ProcessManager>,
    scheduled_task_rx: mpsc::UnboundedReceiver<ScheduledTask>,
  ) -> Self {
    Self { setting, connection_manager, process_manager, scheduled_task_rx }
  }

  /// 启动任务执行器
  pub async fn run_loop(&mut self) {
    info!("Starting TaskExecutor for agent {}", self.setting.agent_id);

    loop {
      match self.scheduled_task_rx.recv().await {
        Some(task) => self.execute_task(task).await,
        None => {
          info!("The scheduled_task_rx channel closed");
          break;
        }
      }
    }
  }

  /// 执行任务
  async fn execute_task(&self, task: ScheduledTask) {
    let instance_id = task.task_instance_id();
    let agent_id = self.setting.agent_id.clone();

    // 执行任务（单次执行）
    if let Err(error) = self._execute_task(task).await {
      let event = Self::process_execution_error(agent_id, instance_id, error);
      if let Err(e) = self.connection_manager.send_event(event).await {
        error!("Failed to send event: {:?}", e);
      }
    }
  }

  /// 执行单次任务尝试
  async fn _execute_task(&self, task: ScheduledTask) -> Result<(), TaskExecutionError> {
    // 使用ProcessManager启动进程
    let instance_id = self
      .process_manager
      .spawn_process(task)
      .await
      .map_err(|_e| TaskExecutionError::ProcessStartFailed)?;

    info!("Started process for task instance {}", instance_id);
    Ok(())
  }

  fn process_execution_error(agent_id: String, instance_id: Uuid, error: TaskExecutionError) -> WebSocketEvent {
    let mut payload = TaskInstanceChanged {
      instance_id,
      agent_id,
      timestamp: now_epoch_millis(),
      data: None,
      error_message: None,
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
    WebSocketEvent::new_task_instance_updated(payload)
  }
}
