use std::sync::{Arc, Mutex};
use std::time::Duration;

use fusion_common::time::now_offset;
use fusion_core::DataError;
use fusion_core::timer::TimerRef;
use log::{debug, error, info, warn};
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::time::{Instant, interval};

use hetuflow_core::protocol::{ScheduledTask, TaskRequest, TaskResponse, WebSocketEvent};
use hetuflow_core::types::{EventKind, HetuflowCommand};

use crate::service::ConnectionManager;
use crate::setting::HetuflowAgentSetting;

/// 任务调度状态
#[derive(Debug, Clone)]
pub struct TaskSchedulingState {
  /// 活跃任务数
  pub active_tasks: usize,
  /// 当前负载因子
  pub load_factor: f64,
  /// 可用容量
  pub available_capacity: usize,
  /// 最后轮询时间
  pub last_poll_time: Option<Instant>,
  /// 轮询计数
  pub poll_count: u64,
}

impl Default for TaskSchedulingState {
  fn default() -> Self {
    Self { active_tasks: 0, load_factor: 0.0, available_capacity: 0, last_poll_time: None, poll_count: 0 }
  }
}

/// 任务调度器。负责 Agent Poll 机制、任务调度和容量计算。
///
/// 向 Server 请求 Poll 任务执行
pub struct TaskScheduler {
  /// 配置
  setting: Arc<HetuflowAgentSetting>,
  shutdown_tx: broadcast::Sender<()>,
  /// 调度状态
  state: Arc<RwLock<TaskSchedulingState>>,

  connection_manager: Arc<ConnectionManager>,
  timer_ref: TimerRef,
  scheduled_task_tx: kanal::AsyncSender<ScheduledTask>,
}

impl TaskScheduler {
  /// 创建新的任务调度器
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    shutdown_tx: broadcast::Sender<()>,
    connection_manager: Arc<ConnectionManager>,
    timer_ref: TimerRef,
    scheduled_task_tx: kanal::AsyncSender<ScheduledTask>,
  ) -> Self {
    Self {
      setting,
      shutdown_tx,
      state: Arc::new(RwLock::new(TaskSchedulingState::default())),
      connection_manager,
      timer_ref,
      scheduled_task_tx,
    }
  }

  /// 获取调度状态
  pub async fn get_state(&self) -> TaskSchedulingState {
    let state = self.state.read().await;
    state.clone()
  }

  /// 获取负载因子
  pub async fn get_load_factor(&self) -> f64 {
    let state = self.state.read().await;
    state.load_factor
  }

  /// 启动任务调度器
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Starting TaskScheduler");

    // 启动轮询循环
    let poll_state = self.state.clone();
    let setting = self.setting.clone();
    let connection_manager = self.connection_manager.clone();
    let shutdown_rx = self.shutdown_tx.subscribe();
    tokio::spawn(Self::polling_task_loop(setting, shutdown_rx, poll_state, connection_manager));

    self.start_task_run_loop();

    info!("TaskScheduler started successfully");
    Ok(())
  }

  fn start_task_run_loop(&self) {
    let command_rx = self.connection_manager.subscribe_command();
    tokio::spawn(Self::process_task_response_loop(
      self.scheduled_task_tx.clone_sync(),
      self.shutdown_tx.clone(),
      self.timer_ref.clone(),
      command_rx,
    ));
  }

  /// 轮询请求待执行任务循环
  async fn polling_task_loop(
    setting: Arc<HetuflowAgentSetting>,
    mut shutdown_rx: broadcast::Receiver<()>,
    state: Arc<RwLock<TaskSchedulingState>>,
    connection_manager: Arc<ConnectionManager>,
  ) {
    info!("TaskScheduler polling loop started");

    let mut poll_interval = interval(Duration::from_secs(setting.polling.interval_seconds));

    loop {
      tokio::select! {
        _ = poll_interval.tick() => {},
        _ = shutdown_rx.recv() => {
          info!("TaskScheduler polling loop stopped");
          return;
        }
      };

      // 更新轮询状态
      let (current_capacity, current_load_factor) = {
        let mut state_guard = state.write().await;
        state_guard.last_poll_time = Some(Instant::now());
        state_guard.poll_count += 1;
        (state_guard.available_capacity, state_guard.load_factor)
      };

      // 检查是否需要轮询新任务
      if current_capacity > 0 && current_load_factor < setting.polling.load_factor_threshold {
        debug!(
          "Polling for new tasks, available capacity: {}, load factor: {:.2}",
          current_capacity, current_load_factor
        );

        // 发送轮询请求
        let poll_request = TaskRequest {
          agent_id: setting.agent_id,
          max_tasks: current_capacity as u32,
          available_capacity: current_capacity as u32,
          tags: setting.tags.clone(),
        };

        if let Err(e) = connection_manager.send_event(WebSocketEvent::new(EventKind::PollTaskRequest, poll_request)) {
          error!("Failed to send poll request: {}", e);
        }
      } else {
        debug!("Skipping poll - capacity: {}, load factor: {:.2}", current_capacity, current_load_factor);
      }
    }
  }

  async fn process_task_response_loop(
    scheduled_task_tx: kanal::Sender<ScheduledTask>,
    shutdown_tx: broadcast::Sender<()>,
    mut timer_ref: TimerRef,
    mut command_rx: broadcast::Receiver<HetuflowCommand>,
  ) {
    let mut shutdown_rx = shutdown_tx.subscribe();
    loop {
      tokio::select! {
        command = command_rx.recv() => {
          match command {
            Ok(HetuflowCommand::DispatchTask(task_poll_resp)) => {
              for task in task_poll_resp.tasks.iter().cloned() {
                let start_at = &task.task_instance.started_at;
                let timeout = start_at.signed_duration_since(now_offset()).to_std().unwrap_or(Duration::ZERO);

                let tx = scheduled_task_tx.clone();
                timer_ref.schedule_action_once(task.task_instance_id(), timeout, move |task_instance_id| {
                  // 发送到 TaskExecutor ，由 TaskExecutor 执行任务
                  if let Err(e) = tx.send(task  ) {
                    warn!("Failed to send task to TaskExecutor. TaskInstanceId: {}, Error: {}", task_instance_id, e);
                  }
                });
              }
            }
            Ok(_) => {
              debug!("Command that doesn't care");
            }
            Err(e) => {
              error!("Failed to receive command: {}", e);
              return;
            }
          }
        }
        _ = shutdown_rx.recv() => {
          info!("process_task_response_loop stopped");
          return;
        }
      }
    }
  }
}
