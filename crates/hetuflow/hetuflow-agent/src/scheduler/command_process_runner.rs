use std::sync::Arc;
use std::time::Duration;

use fusion_common::time::now_offset;
use fusion_core::{DataError, concurrent::ServiceTask, timer::TimerRef};
use log::{debug, error, info, warn};
use mea::mpsc;
use mea::shutdown::ShutdownRecv;
use tokio::sync::broadcast::{self, error::RecvError};

use hetuflow_core::{
  protocol::{CommandMessage, ScheduledTask},
  types::CommandKind,
};

use crate::setting::HetuflowAgentSetting;

pub struct CommandProcessRunner {
  setting: Arc<HetuflowAgentSetting>,
  scheduled_task_tx: mpsc::UnboundedSender<ScheduledTask>,
  shutdown_rx: ShutdownRecv,
  timer_ref: TimerRef,
  command_rx: broadcast::Receiver<CommandMessage>,
}

impl ServiceTask<()> for CommandProcessRunner {
  async fn run_loop(&mut self) -> Result<(), DataError> {
    loop {
      tokio::select! {
        command = self.command_rx.recv() => {
          match command {
            Ok(cmd) => {
              self.process_command(cmd);
            }
            Err(RecvError::Closed) => {
              error!("CommandMessage receive channel has been closed.");
              break;
            }
            Err(RecvError::Lagged(count)) => {
              warn!("CommandMessage receive channel lagged {} messages.", count);
            }
          }
        }
        _ = self.shutdown_rx.is_shutdown() => {
          info!("ScheduleTaskRunner stopped");
          break;
        }
      }
    }
    Ok(())
  }
}

impl CommandProcessRunner {
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    scheduled_task_tx: mpsc::UnboundedSender<ScheduledTask>,
    shutdown_rx: ShutdownRecv,
    timer_ref: TimerRef,
    command_rx: broadcast::Receiver<CommandMessage>,
  ) -> Self {
    Self { setting, scheduled_task_tx, shutdown_rx, timer_ref, command_rx }
  }

  fn process_command(&mut self, cmd: CommandMessage) {
    match cmd.kind() {
      CommandKind::TaskAcquired => {
        let response = cmd.as_acquire_task().unwrap();
        for task in response.tasks.iter().cloned() {
          let scheduled_at = &task.task.scheduled_at;
          let timeout = scheduled_at.signed_duration_since(now_offset()).to_std().unwrap_or(Duration::ZERO);

          let tx = self.scheduled_task_tx.clone();
          self.timer_ref.schedule_action_once(task.task_instance_id(), timeout, move |task_instance_id| {
            // 发送到 TaskExecutor ，由 TaskExecutor 执行任务
            if let Err(e) = tx.send(task) {
              warn!("Failed to send task to TaskExecutor. TaskInstanceId: {}, Error: {}", task_instance_id, e);
            }
          });
        }
      }
      CommandKind::AgentRegistered => {
        let resp = cmd.as_agent_registered().unwrap();
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
      _ => {
        debug!("Command that doesn't care: {:?}", cmd);
      }
    }
  }
}
