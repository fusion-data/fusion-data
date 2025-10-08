use std::process::Stdio;
use std::sync::Arc;

use fusion_common::time::now_epoch_millis;
use fusion_core::DataError;
use log::{error, info, warn};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::broadcast;
use uuid::Uuid;

use hetuflow_core::protocol::{
  AgentLogMessage, EventMessage, LogKind, ProcessEvent, ProcessEventKind, ProcessInfo, ProcessStatus, ScheduledTask,
};

use crate::connection::ConnectionManager;
use crate::setting::ProcessSetting;

use super::{ActiveProcesses, ProcessItem, spawn_kill_process};

/// 进程管理器。负责管理活跃进程、处理进程事件和后台清理僵尸进程
pub struct ProcessManager {
  pub(super) config: Arc<ProcessSetting>,
  connection_manager: Arc<ConnectionManager>,
  /// active processes, key: task instance id, value: process info
  pub(super) active_processes: ActiveProcesses,
  /// Process event broadcaster
  pub(super) event_broadcaster: broadcast::Sender<ProcessEvent>,
}

impl ProcessManager {
  /// 创建新的进程管理器
  pub fn new(process_config: Arc<ProcessSetting>, connection_manager: Arc<ConnectionManager>) -> Self {
    let (event_broadcaster, _) = broadcast::channel(1000);

    Self { config: process_config, connection_manager, active_processes: Default::default(), event_broadcaster }
  }

  /// Start a new process
  pub async fn spawn_process(&self, task: ScheduledTask) -> Result<Uuid, DataError> {
    let instance_id = task.task_instance_id();

    // Check concurrent process limit
    {
      let active_processes = self.active_processes.read().await;
      if active_processes.len() >= self.config.max_concurrent_processes as usize {
        return Err(DataError::server_error(format!(
          "Maximum concurrent processes limit reached, current: {}, max: {}",
          active_processes.len(),
          self.config.max_concurrent_processes
        )));
      }
      if active_processes.contains_key(&instance_id) {
        return Err(DataError::server_error(format!("Process instance already exists, instance_id: {}", instance_id)));
      }
    }

    // Build command
    let mut cmd = Command::new(task.task.config.cmd.as_ref());
    cmd.args(&task.task.config.args).stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());

    // Set working directory
    let work_dir_path = self.config.run_base_dir()?.join(task.job_id().to_string()).join(task.task_id().to_string());
    if !work_dir_path.exists() {
      tokio::fs::create_dir_all(&work_dir_path).await?;
    }
    cmd.current_dir(work_dir_path);

    // Set environment variables
    for (key, value) in task.task.environments() {
      cmd.env(key, value);
    }

    // Set process group (Only Unix system)
    #[cfg(unix)]
    {
      cmd.process_group(0);
    }

    info!("Spawning process, instance id: {}, cmd: {:?}", instance_id, cmd);

    // Spawn process
    let mut child = cmd.spawn()?;
    let pid: u32 = child.id().ok_or_else(|| DataError::server_error("Start process failed"))?;

    self.forward_stdout(&mut child, instance_id);
    self.forward_stderr(&mut child, instance_id);

    // Create process info
    let info = ProcessInfo {
      pid,
      instance_id,
      status: ProcessStatus::Running,
      started_at: now_epoch_millis(),
      completed_at: None,
      exit_code: None,
    };

    // Send process start event
    let _ = self.event_broadcaster.send(ProcessEvent::new_with_data(
      instance_id,
      ProcessEventKind::Running,
      serde_json::to_string(&info).ok(),
    ));

    // Store active process info
    {
      let mut active_processes = self.active_processes.write().await;
      active_processes.insert(instance_id, ProcessItem { info });
    }

    // Wait for process to exit
    let active_processes = self.active_processes.clone();
    let event_broadcaster = self.event_broadcaster.clone();
    tokio::spawn(async move {
      let exit_code = match child.wait().await {
        Ok(exit_status) => exit_status.code().unwrap_or(-1),
        Err(e) => {
          error!("Error waiting for process instance_id: {}, error: {}", instance_id, e);
          -1
        }
      };
      if let Some(mut process_item) = active_processes.write().await.remove(&instance_id) {
        if exit_code == 0 {
          process_item.info.status = ProcessStatus::Succeed;
        } else {
          process_item.info.status = ProcessStatus::Failed;
        };
        process_item.info.exit_code = Some(exit_code);
        process_item.info.completed_at = Some(now_epoch_millis());
        let _ = event_broadcaster.send(ProcessEvent::new_with_data(
          instance_id,
          ProcessEventKind::Exited,
          serde_json::to_string(&process_item.info).ok(),
        ));
      }
    });

    info!("Process spawned successfully, instance id: {}, PID: {}", instance_id, pid);
    Ok(instance_id)
  }

  fn forward_stderr(&self, child: &mut Child, instance_id: Uuid) {
    // Forward process stderr
    if let Some(stderr) = child.stderr.take() {
      let connection_manager = self.connection_manager.clone();
      tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut sequence = 0;
        loop {
          let mut line = String::new();
          match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
              let trimmed = line.trim_end();
              sequence += 1;
              let _ = connection_manager
                .send_event(EventMessage::new_log_message(AgentLogMessage::new(
                  instance_id,
                  sequence,
                  LogKind::Stderr,
                  trimmed.to_string(),
                )))
                .await;
            }
            Err(e) => {
              warn!("Error reading stdout: {}", e);
              break;
            }
          }
        }
      });
    }
  }

  fn forward_stdout(&self, child: &mut Child, instance_id: Uuid) {
    // Forward process stdout
    if let Some(stdout) = child.stdout.take() {
      let connection_manager = self.connection_manager.clone();
      tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        let mut sequence = 0;
        loop {
          let mut line = String::new();
          match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
              let trimmed = line.trim_end();
              sequence += 1;
              let _ = connection_manager
                .send_event(EventMessage::new_log_message(AgentLogMessage::new(
                  instance_id,
                  sequence,
                  LogKind::Stdout,
                  trimmed.to_string(),
                )))
                .await;
            }
            Err(e) => {
              warn!("Error reading stdout: {}", e);
              break;
            }
          }
        }
      });
    }
  }

  /// 强制终止进程
  /// 终止进程，返回 JoinHandle 用于异步处理
  pub fn spawn_kill_process(&self, instance_id: Uuid) -> tokio::task::JoinHandle<Result<Option<Uuid>, DataError>> {
    let active_processes = self.active_processes.clone();
    let event_broadcaster = self.event_broadcaster.clone();
    spawn_kill_process(instance_id, active_processes, event_broadcaster)
  }

  pub async fn kill_all_processes(&self) -> Result<(), DataError> {
    let handles = self
      .active_processes
      .read()
      .await
      .keys()
      .map(|instance_id| (*instance_id, self.spawn_kill_process(*instance_id)))
      .collect::<Vec<_>>();

    // Wait for all kill process tasks to complete
    for (instance_id, handle) in handles {
      if let Err(e) = handle.await {
        warn!("Error waiting for kill process task for {}, error: {}", instance_id, e);
      }
    }

    Ok(())
  }

  /// 获取进程信息
  pub async fn get_process_info(&self, instance_id: &Uuid) -> Option<ProcessInfo> {
    let active_processes = self.active_processes.read().await;
    active_processes.get(instance_id).map(|item| item.info.clone())
  }

  /// 获取所有活跃进程
  pub async fn get_active_processes(&self) -> Vec<ProcessInfo> {
    let active_processes = self.active_processes.read().await;
    active_processes.values().map(|item| item.info.clone()).collect()
  }

  /// 获取可用容量
  pub async fn available_capacity(&self) -> u32 {
    let active_processes = self.active_processes.read().await;
    self.config.max_concurrent_processes - active_processes.len() as u32
  }

  /// 获取事件接收器
  pub fn subscribe_events(&self) -> broadcast::Receiver<ProcessEvent> {
    self.event_broadcaster.subscribe()
  }
}
