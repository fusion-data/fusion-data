use std::sync::Arc;

use hetuflow_core::{
  protocol::{ProcessEvent, ProcessEventKind, TaskInstanceChanged, WebSocketEvent},
  types::TaskInstanceStatus,
};
use log::{error, info};
use mea::shutdown::ShutdownRecv;

use crate::{connection::ConnectionManager, process::ProcessManager, setting::HetuflowAgentSetting};

pub struct ProcessEventRunner {
  setting: Arc<HetuflowAgentSetting>,
  connection_manager: Arc<ConnectionManager>,
  process_manager: Arc<ProcessManager>,
  shutdown_rx: ShutdownRecv,
}

impl ProcessEventRunner {
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    connection_manager: Arc<ConnectionManager>,
    process_manager: Arc<ProcessManager>,
    shutdown_rx: ShutdownRecv,
  ) -> Self {
    Self { setting, connection_manager, process_manager, shutdown_rx }
  }

  pub async fn run_loop(&self) {
    let mut process_event_rx = self.process_manager.subscribe_events();

    loop {
      tokio::select! {
        event_result = process_event_rx.recv() => {
          match event_result {
            Ok(event) => self.handle_process_event(event).await,
            Err(e) => {
              info!("The process_event_rx channel closed: {}", e);
              break;
            }
          }
        },
        _ = self.shutdown_rx.is_shutdown() => {
          info!("TaskExecutor process_event_rx loop stopped");
          break;
        }
      }
    }
  }

  async fn handle_process_event(&self, event: ProcessEvent) {
    let status = match event.kind {
      ProcessEventKind::Started => TaskInstanceStatus::Running,
      ProcessEventKind::Exited => TaskInstanceStatus::Succeeded,
      ProcessEventKind::Sigterm => TaskInstanceStatus::Failed,
      ProcessEventKind::Sigkill => TaskInstanceStatus::Failed,
      ProcessEventKind::ResourceViolation => TaskInstanceStatus::Failed,
      ProcessEventKind::BecameZombie => TaskInstanceStatus::Failed,
    };
    let event = WebSocketEvent::new_task_instance_updated(TaskInstanceChanged {
      instance_id: event.instance_id,
      agent_id: self.setting.agent_id.clone(),
      timestamp: event.timestamp,
      data: event.data,
      error_message: None,
      metrics: None,
      status,
    });
    if let Err(e) = self.connection_manager.send_event(event).await {
      error!("Failed to send event with process event. error: {:?}", e);
    }
  }
}
