use std::sync::{Arc, Mutex};
use std::time::Duration;

use fusion_common::time::now_offset;
use fusion_core::DataError;
use fusion_core::timer::TimerRef;
use log::{debug, error, info, warn};
use mea::shutdown::ShutdownRecv;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio::time::interval;

use hetuflow_core::protocol::{AcquireTaskRequest, ScheduledTask, WebSocketEvent};
use hetuflow_core::types::{EventKind, HetuflowCommand};

use crate::service::{ConnectionManager, ProcessManager};
use crate::setting::HetuflowAgentSetting;

/// 任务调度器。负责 Agent Poll 机制、任务调度和容量计算。
///
/// 向 Server 请求 Poll 任务执行
pub struct TaskScheduler {
  poll_task_runner: Arc<Mutex<Option<PollTaskRunner>>>,
  schedule_task_runner: Arc<Mutex<Option<ScheduleTaskRunner>>>,
  scheduled_task_rx: kanal::AsyncReceiver<ScheduledTask>,
}

impl TaskScheduler {
  /// 创建新的任务调度器
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    process_manager: Arc<ProcessManager>,
    shutdown_rx: ShutdownRecv,
    connection_manager: Arc<ConnectionManager>,
    timer_ref: TimerRef,
  ) -> Self {
    let (scheduled_task_tx, scheduled_task_rx) = kanal::unbounded_async();
    let command_rx = connection_manager.subscribe_command();
    let schedule_task_runner =
      ScheduleTaskRunner { scheduled_task_tx, shutdown_rx: shutdown_rx.clone(), timer_ref, command_rx };
    let schedule_task_runner = Arc::new(Mutex::new(Some(schedule_task_runner)));

    let poll_task_runner = PollTaskRunner { setting, shutdown_rx, process_manager, connection_manager };
    let poll_task_runner = Arc::new(Mutex::new(Some(poll_task_runner)));

    Self { poll_task_runner, schedule_task_runner, scheduled_task_rx }
  }

  /// 获取 ScheduledTask Receiver
  pub fn scheduled_task_rx(&self) -> kanal::AsyncReceiver<ScheduledTask> {
    self.scheduled_task_rx.clone()
  }

  /// 启动任务调度器
  pub fn start(&self) -> Result<Vec<JoinHandle<()>>, DataError> {
    info!("Starting TaskScheduler");

    let h1 = self.start_poll_task_runner();
    let h2 = self.start_task_run_loop();

    info!("TaskScheduler started successfully");
    Ok(vec![h1, h2])
  }

  fn start_poll_task_runner(&self) -> JoinHandle<()> {
    let mut guard = self.poll_task_runner.lock().unwrap();
    if let Some(mut poll_task_runner) = guard.take() {
      tokio::spawn(async move { poll_task_runner.run_loop().await })
    } else {
      panic!("poll_task_runner is None")
    }
  }

  fn start_task_run_loop(&self) -> JoinHandle<()> {
    let mut guard = self.schedule_task_runner.lock().unwrap();
    if let Some(mut schedule_task_runner) = guard.take() {
      tokio::spawn(async move { schedule_task_runner.run_loop().await })
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
      tokio::select! {
        command = self.command_rx.recv() => {
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
              break;
            }
          }
        }
        _ = self.shutdown_rx.is_shutdown() => {
          info!("ScheduleTaskRunner stopped");
          break;
        }
      }
    }
    let _ = self.scheduled_task_tx.close();
  }
}

struct PollTaskRunner {
  setting: Arc<HetuflowAgentSetting>,
  shutdown_rx: ShutdownRecv,
  process_manager: Arc<ProcessManager>,
  connection_manager: Arc<ConnectionManager>,
}
impl PollTaskRunner {
  /// 轮询请求待执行任务循环
  async fn run_loop(&mut self) {
    let mut poll_interval = interval(Duration::from_secs(self.setting.polling.interval_seconds));

    loop {
      tokio::select! {
        _ = poll_interval.tick() => {},
        _ = self.shutdown_rx.is_shutdown() => {
          info!("PollTaskRunner polling loop stopped");
          return;
        }
      };

      let acquire_count = self.process_manager.available_capacity().await;
      // 检查是否需要轮询新任务
      if acquire_count > 0 {
        debug!("Polling for new tasks, acquire_count: {}", acquire_count);

        // 发送轮询请求
        let poll_request = AcquireTaskRequest {
          agent_id: self.setting.agent_id,
          max_tasks: self.setting.process.max_concurrent_processes,
          acquire_count,
          labels: self.setting.labels.clone(),
        };

        if let Err(e) =
          self.connection_manager.send_event(WebSocketEvent::new(EventKind::PollTaskRequest, poll_request))
        {
          error!("Failed to send poll request: {}", e);
        }
      } else {
        debug!("Skipping poll - acquire_count: {}", acquire_count);
      }
    }
  }
}
