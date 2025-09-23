use std::sync::Arc;

use hetuflow_core::protocol::{AcquireTaskRequest, WebSocketEvent};
use log::{debug, error, info};
use mea::shutdown::ShutdownRecv;

use crate::{connection::ConnectionManager, process::ProcessManager, setting::HetuflowAgentSetting};

pub struct PollTaskRunner {
  setting: Arc<HetuflowAgentSetting>,
  connection_manager: Arc<ConnectionManager>,
  process_manager: Arc<ProcessManager>,
  shutdown_rx: ShutdownRecv,
}

impl PollTaskRunner {
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    connection_manager: Arc<ConnectionManager>,
    process_manager: Arc<ProcessManager>,
    shutdown_rx: ShutdownRecv,
  ) -> Self {
    Self { setting, connection_manager, process_manager, shutdown_rx }
  }

  /// Scheduled polling request task
  pub async fn run_loop(&self) {
    let mut poll_interval = tokio::time::interval(self.setting.polling.interval);

    loop {
      tokio::select! {
        _ = poll_interval.tick() => self.attempt_poll().await,
        _ = self.shutdown_rx.is_shutdown() => {
          info!("PollTaskRunner polling loop stopped");
          break;
        }
      };
    }
  }

  async fn attempt_poll(&self) {
    let acquire_count = self.process_manager.available_capacity().await;
    // Check if polling for new tasks is required
    if acquire_count == 0 {
      return;
    }

    debug!("Polling for new tasks, acquire_count: {}", acquire_count);
    let poll_request = AcquireTaskRequest {
      agent_id: self.setting.agent_id.clone(),
      max_tasks: self.setting.process.max_concurrent_processes,
      acquire_count,
      labels: self.setting.labels.clone(),
    };
    let event = WebSocketEvent::new_poll_task(poll_request);
    if let Err(e) = self.connection_manager.send_event(event).await {
      error!("Failed to send poll request: {}", e);
    }
  }
}
