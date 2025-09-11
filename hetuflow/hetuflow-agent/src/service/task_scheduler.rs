use std::sync::{Arc, Mutex};
use std::time::Duration;

use fusion_common::time::now_offset;
use fusion_core::DataError;
use fusion_core::timer::TimerRef;
use futures_util::{FutureExt, pin_mut};
use log::{debug, error, info, warn};
use mea::shutdown::ShutdownRecv;
use tokio::sync::{RwLock, broadcast};
use tokio::time::{Instant, interval};

use hetuflow_core::protocol::{AcquireTaskRequest, ScheduledTask, WebSocketEvent};
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
  poll_task_runner: Arc<Mutex<Option<PollTaskRunner>>>,
  schedule_task_runner: Arc<Mutex<Option<ScheduleTaskRunner>>>,
  /// 调度状态
  state: Arc<RwLock<TaskSchedulingState>>,
}

impl TaskScheduler {
  /// 创建新的任务调度器
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    shutdown_rx: ShutdownRecv,
    connection_manager: Arc<ConnectionManager>,
    timer_ref: TimerRef,
    scheduled_task_tx: kanal::AsyncSender<ScheduledTask>,
  ) -> Self {
    let state = Arc::new(RwLock::new(TaskSchedulingState::default()));

    let command_rx = connection_manager.subscribe_command();
    let schedule_task_runner =
      ScheduleTaskRunner { scheduled_task_tx, shutdown_rx: shutdown_rx.clone(), timer_ref, command_rx };
    let schedule_task_runner = Arc::new(Mutex::new(Some(schedule_task_runner)));

    let poll_task_runner = PollTaskRunner { setting, shutdown_rx, state: state.clone(), connection_manager };
    let poll_task_runner = Arc::new(Mutex::new(Some(poll_task_runner)));

    Self { poll_task_runner, schedule_task_runner, state }
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

    self.start_poll_task_runner();

    self.start_task_run_loop();

    info!("TaskScheduler started successfully");
    Ok(())
  }

  fn start_poll_task_runner(&self) {
    let mut guard = self.poll_task_runner.lock().unwrap();
    if let Some(mut poll_task_runner) = guard.take() {
      tokio::spawn(async move { poll_task_runner.run_loop().await });
    } else {
      panic!("poll_task_runner is None");
    }
  }

  fn start_task_run_loop(&self) {
    let mut guard = self.schedule_task_runner.lock().unwrap();
    if let Some(mut schedule_task_runner) = guard.take() {
      tokio::spawn(async move { schedule_task_runner.run_loop().await });
    } else {
      panic!("schedule_task_runner is None");
    }
  }
}

struct ScheduleTaskRunner {
  scheduled_task_tx: kanal::AsyncSender<ScheduledTask>,
  shutdown_rx: ShutdownRecv,
  timer_ref: TimerRef,
  command_rx: broadcast::Receiver<HetuflowCommand>,
}
impl ScheduleTaskRunner {
  async fn run_loop(&mut self) {
    loop {
      let command_rx_fut = self.command_rx.recv().fuse();
      let shutdown_rx_fut = self.shutdown_rx.is_shutdown().fuse();
      pin_mut!(command_rx_fut, shutdown_rx_fut);
      tokio::select! {
        command = command_rx_fut => {
          match command {
            Ok(HetuflowCommand::AcquiredTask(task_poll_resp)) => {
              for task in task_poll_resp.tasks.iter().cloned() {
                let start_at = &task.task_instance.started_at;
                let timeout = start_at.signed_duration_since(now_offset()).to_std().unwrap_or(Duration::ZERO);

                let tx = self.scheduled_task_tx.clone_sync();
                self.timer_ref.schedule_action_once(task.task_instance_id(), timeout, move |task_instance_id| {
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
        _ = shutdown_rx_fut => {
          info!("process_task_response_loop stopped");
          return;
        }
      }
    }
  }
}

struct PollTaskRunner {
  setting: Arc<HetuflowAgentSetting>,
  shutdown_rx: ShutdownRecv,
  state: Arc<RwLock<TaskSchedulingState>>,
  connection_manager: Arc<ConnectionManager>,
}
impl PollTaskRunner {
  /// 轮询请求待执行任务循环
  async fn run_loop(&mut self) {
    let mut poll_interval = interval(Duration::from_secs(self.setting.polling.interval_seconds));

    loop {
      let poll_interval_fut = poll_interval.tick().fuse();
      let shutdown_rx_fut = self.shutdown_rx.is_shutdown().fuse();
      pin_mut!(poll_interval_fut, shutdown_rx_fut);
      tokio::select! {
        _ = poll_interval_fut => {},
        _ = shutdown_rx_fut => {
          info!("TaskScheduler polling loop stopped");
          return;
        }
      };

      // 更新轮询状态
      let (current_capacity, current_load_factor) = {
        let mut state_guard = self.state.write().await;
        state_guard.last_poll_time = Some(Instant::now());
        state_guard.poll_count += 1;
        (state_guard.available_capacity, state_guard.load_factor)
      };

      // 检查是否需要轮询新任务
      if current_capacity > 0 && current_load_factor < self.setting.polling.load_factor_threshold {
        debug!(
          "Polling for new tasks, available capacity: {}, load factor: {:.2}",
          current_capacity, current_load_factor
        );

        // 发送轮询请求
        let poll_request = AcquireTaskRequest {
          agent_id: self.setting.agent_id,
          max_tasks: current_capacity as u32,
          available_capacity: current_capacity as u32,
          tags: self.setting.tags.clone(),
        };

        if let Err(e) =
          self.connection_manager.send_event(WebSocketEvent::new(EventKind::PollTaskRequest, poll_request))
        {
          error!("Failed to send poll request: {}", e);
        }
      } else {
        debug!("Skipping poll - capacity: {}, load factor: {:.2}", current_capacity, current_load_factor);
      }
    }
  }
}
