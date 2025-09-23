use std::sync::Arc;

use hetuflow_core::types::HetuflowCommand;
use log::{error, info};
use mea::shutdown::ShutdownRecv;
use tokio::sync::broadcast;

use crate::connection::ConnectionManager;

pub struct CommandRunner {
  connection_manager: Arc<ConnectionManager>,
  shutdown_rx: ShutdownRecv,
}

impl CommandRunner {
  pub fn new(connection_manager: Arc<ConnectionManager>, shutdown_rx: ShutdownRecv) -> Self {
    Self { connection_manager, shutdown_rx }
  }

  pub async fn run_loop(&self) {
    let mut command_rx = self.connection_manager.subscribe_command();
    loop {
      tokio::select! {
        command = command_rx.recv() => {
          match command {
            Ok(command) => {
              match command {
                HetuflowCommand::AcquiredTask(task_response) => {
                  if let Err(e) = self.connection_manager.publish_acquire_task(task_response) {
                    error!("Failed to send acquired task to TaskScheduler. Error: {}", e);
                  }
                }
                _ => {
                  // 其他命令暂不处理
                }
              }
            }
            Err(e) => {
              error!("Failed to receive command. Error: {}", e);
              return;
            }
          }
        }
        _ = self.shutdown_rx.is_shutdown() => {
          info!("CommandRunner exited.");
          return;
        }
      }
    }
  }
}
