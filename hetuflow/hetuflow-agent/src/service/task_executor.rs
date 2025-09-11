use std::sync::Arc;

use fusion_common::time::now_epoch_millis;
use fusion_core::DataError;
use futures_util::{FutureExt, pin_mut};
use log::{error, info};
use mea::shutdown::ShutdownRecv;
use tokio::task::JoinHandle;
use uuid::Uuid;

use hetuflow_core::{
  protocol::{ProcessEvent, ProcessEventKind, ScheduledTask, TaskExecutionError, TaskInstanceUpdated, WebSocketEvent},
  types::{EventKind, TaskInstanceStatus},
};

use crate::{
  service::{ConnectionManager, ProcessManager},
  setting::HetuflowAgentSetting,
};

/// 任务执行器。负责执行具体的任务，包括进程管理和状态上报
pub struct TaskExecutor {
  setting: Arc<HetuflowAgentSetting>,
  process_manager: Arc<ProcessManager>,
  connection_manager: Arc<ConnectionManager>,
  scheduled_task_rx: kanal::AsyncReceiver<ScheduledTask>,
  shutdown_rx: std::sync::Mutex<Option<ShutdownRecv>>,
}

impl TaskExecutor {
  /// 创建新的任务执行器
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    process_manager: Arc<ProcessManager>,
    connection_manager: Arc<ConnectionManager>,
    scheduled_task_rx: kanal::AsyncReceiver<ScheduledTask>,
    shutdown_rx: ShutdownRecv,
  ) -> Self {
    let shutdown_rx = std::sync::Mutex::new(Some(shutdown_rx));
    Self { setting, process_manager, connection_manager, scheduled_task_rx, shutdown_rx }
  }

  pub fn start(&self) -> Result<Vec<JoinHandle<()>>, DataError> {
    info!("Starting TaskExecutor");
    let h1 = self.run_scheduled_task_loop();
    let h2 = self.run_process_event_loop();

    // discard ShutdownRecv
    let _ = self.shutdown_rx.lock().unwrap().take();

    info!("TaskExecutor started successfully");
    Ok(vec![h1, h2])
  }

  /// 启动任务执行器
  fn run_scheduled_task_loop(&self) -> JoinHandle<()> {
    info!("Starting TaskExecutor for agent {}", self.setting.agent_id);

    let connection_manager = self.connection_manager.clone();
    let process_manager = self.process_manager.clone();
    let setting = self.setting.clone();
    let scheduled_task_rx = self.scheduled_task_rx.clone();

    tokio::spawn(async move {
      loop {
        match scheduled_task_rx.recv().await {
          Ok(task) => {
            Self::execute_task(setting.clone(), process_manager.clone(), connection_manager.clone(), task).await
          }
          Err(e) => {
            info!("The scheduled_task_rx channel closed: {}", e);
            break;
          }
        }
      }
    })
  }

  fn run_process_event_loop(&self) -> JoinHandle<()> {
    let mut process_event_rx = self.process_manager.subscribe_events();
    let agent_id = self.setting.agent_id;
    let connection_manager = self.connection_manager.clone();
    let shutdown_rx = self.shutdown_rx.lock().unwrap().clone().unwrap();

    tokio::spawn(async move {
      loop {
        let process_event_rx_fut = process_event_rx.recv().fuse();
        let shutdown_rx_fut = shutdown_rx.is_shutdown().fuse();
        pin_mut!(process_event_rx_fut, shutdown_rx_fut);
        futures_util::select! {
          event_result = process_event_rx_fut => {
            match event_result {
              Ok(event) => Self::handle_process_event(agent_id, connection_manager.clone(), event).await,
              Err(e) => {
                info!("The process_event_rx channel closed: {}", e);
                break;
              }
            }
          },
          _ = shutdown_rx_fut => {
            info!("TaskExecutor process_event_rx loop stopped");
            break;
          }
        }
      }
    })
  }

  /// 停止任务执行器
  pub async fn stop(&self) -> Result<(), DataError> {
    info!("Stopping TaskExecutor");
    self.process_manager.kill_all_processes().await?;
    info!("TaskExecutor stopped");
    Ok(())
  }

  async fn handle_process_event(agent_id: Uuid, connection_manager: Arc<ConnectionManager>, event: ProcessEvent) {
    let status = match event.kind {
      ProcessEventKind::Started => TaskInstanceStatus::Running,
      ProcessEventKind::Exited => TaskInstanceStatus::Succeeded,
      ProcessEventKind::Sigterm => TaskInstanceStatus::Failed,
      ProcessEventKind::Sigkill => TaskInstanceStatus::Failed,
      ProcessEventKind::ResourceViolation => TaskInstanceStatus::Failed,
      ProcessEventKind::BecameZombie => TaskInstanceStatus::Failed,
    };
    let event = WebSocketEvent::new(
      EventKind::TaskChangedEvent,
      TaskInstanceUpdated {
        instance_id: event.instance_id,
        agent_id,
        timestamp: event.timestamp,
        data: event.data,
        error_message: None,
        metrics: None,
        status,
      },
    );
    if let Err(e) = connection_manager.send_event(event) {
      error!("Failed to send event with process event. error: {:?}", e);
    }
  }

  /// 执行任务
  async fn execute_task(
    setting: Arc<HetuflowAgentSetting>,
    process_manager: Arc<ProcessManager>,
    connection_manager: Arc<ConnectionManager>,
    task: ScheduledTask,
  ) {
    let instance_id = task.task_instance_id();
    let agent_id = setting.agent_id;

    // 执行任务（单次执行）
    if let Err(error) = Self::_execute_task(process_manager, task).await {
      let event = Self::process_execution_error(agent_id, instance_id, error);
      if let Err(e) = connection_manager.send_event(event) {
        error!("Failed to send event: {:?}", e);
      }
    }
  }

  /// 执行单次任务尝试
  async fn _execute_task(process_manager: Arc<ProcessManager>, task: ScheduledTask) -> Result<(), TaskExecutionError> {
    // 获取任务配置
    let task_config = task.task.config.as_ref().ok_or(TaskExecutionError::ConfigurationError)?;

    // 准备环境变量
    let environment =
      if let Some(value) = task.task.environment.clone() { serde_json::from_value(value).ok() } else { None };

    // 使用ProcessManager启动进程
    let instance_id = process_manager
      .spawn_process(
        task.task_instance_id(),
        &task_config.cmd,
        &task_config.args,
        task_config.working_directory.as_deref(),
        environment.as_ref(),
        task_config.resource_limits.as_ref(),
      )
      .await
      .map_err(|_e| TaskExecutionError::ProcessStartFailed)?;

    info!("Started process for task instance {}", instance_id);
    Ok(())
  }

  fn process_execution_error(agent_id: Uuid, instance_id: Uuid, error: TaskExecutionError) -> WebSocketEvent {
    let mut payload = TaskInstanceUpdated {
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
    WebSocketEvent::new(EventKind::TaskChangedEvent, payload)
  }
}
