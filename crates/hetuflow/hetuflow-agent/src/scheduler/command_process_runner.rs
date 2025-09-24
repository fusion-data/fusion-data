use std::sync::Arc;
use std::time::Duration;

use fusion_common::time::now_offset;
use fusion_core::timer::TimerRef;
use log::{debug, error, info, warn};
use mea::mpsc;
use mea::shutdown::ShutdownRecv;
use tokio::sync::broadcast;

use hetuflow_core::protocol::ScheduledTask;
use hetuflow_core::types::HetuflowCommand;

use crate::setting::HetuflowAgentSetting;

pub struct CommandProcessRunner {
  setting: Arc<HetuflowAgentSetting>,
  scheduled_task_tx: mpsc::UnboundedSender<ScheduledTask>,
  shutdown_rx: ShutdownRecv,
  timer_ref: TimerRef,
  command_rx: broadcast::Receiver<HetuflowCommand>,
}

impl CommandProcessRunner {
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    scheduled_task_tx: mpsc::UnboundedSender<ScheduledTask>,
    shutdown_rx: ShutdownRecv,
    timer_ref: TimerRef,
    command_rx: broadcast::Receiver<HetuflowCommand>,
  ) -> Self {
    Self { setting, scheduled_task_tx, shutdown_rx, timer_ref, command_rx }
  }

  pub async fn run_loop(&mut self) {
    loop {
      tokio::select! {
        command = self.command_rx.recv() => {
          match command {
            Ok(HetuflowCommand::AcquiredTask(task_poll_resp)) => {
              for task in task_poll_resp.tasks.iter().cloned() {
                let start_at = &task.task_instance.started_at;
                let timeout = start_at.signed_duration_since(now_offset()).to_std().unwrap_or(Duration::ZERO);

                let tx = self.scheduled_task_tx.clone();
                self.timer_ref.schedule_action_once(task.task_instance_id(), timeout, move |task_instance_id| {
                  // 发送到 TaskExecutor ，由 TaskExecutor 执行任务
                  if let Err(e) = tx.send(task) {
                    warn!("Failed to send task to TaskExecutor. TaskInstanceId: {}, Error: {}", task_instance_id, e);
                  }
                });
              }
            }
            Ok(HetuflowCommand::AgentRegistered(resp)) => {
              if resp.success {
                info!("Agent registered successfully, agent_id: {}", self.setting.agent_id);
              } else {
                error!("Agent registration failed, agent_id: {}, response: {:?}", self.setting.agent_id, resp);
                // Send SIGTERM signal to self to terminate the process
                #[cfg(any(unix, windows))]
                fusion_common::process::send_sigterm_to_self();
                #[cfg(not(any(unix, windows)))]
                panic!("Exit");
              }
            }
            Ok(cmd) => {
              debug!("Command that doesn't care: {}", cmd.as_ref());
            }
            Err(_) => {
              error!("HetuflowCommand receive channel has been closed.");
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
  }
}
