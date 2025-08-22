use std::sync::Arc;

use log::{error, info};
use modelsql::ModelManager;
use tokio::sync::{Mutex, mpsc};

use hetuflow_core::protocol::GatewayCommand;

use crate::gateway::AgentEvent;

use super::{ConnectionManager, GatewayError, MessageHandler};

/// 网关服务
pub struct GatewaySvc {
  connection_manager: Arc<ConnectionManager>,
  message_handler: Arc<MessageHandler>,
  command_receiver: Arc<Mutex<mpsc::UnboundedReceiver<GatewayCommand>>>,
  event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<AgentEvent>>>,
}

impl GatewaySvc {
  pub fn new(
    connection_manager: Arc<ConnectionManager>,
    message_handler: Arc<MessageHandler>,
    command_receiver: mpsc::UnboundedReceiver<GatewayCommand>,
    event_receiver: mpsc::UnboundedReceiver<AgentEvent>,
  ) -> Self {
    Self {
      connection_manager,
      message_handler,
      command_receiver: Arc::new(Mutex::new(command_receiver)),
      event_receiver: Arc::new(Mutex::new(event_receiver)),
    }
  }

  /// 启动网关服务
  pub async fn start(&self) -> Result<(), GatewayError> {
    info!("Starting Gateway Service");

    // 启动命令处理循环
    self.start_command_loop().await?;

    Ok(())
  }

  /// 停止网关服务
  pub async fn stop(&self) -> Result<(), GatewayError> {
    info!("Stopping Gateway Service");
    // 关闭信号会在start的spawn中处理
    Ok(())
  }

  /// 启动命令处理循环
  async fn start_command_loop(&self) -> Result<(), GatewayError> {
    let command_receiver = self.command_receiver.clone();
    let connection_manager = self.connection_manager.clone();

    tokio::spawn(async move {
      let mut receiver = command_receiver.lock().await;
      while let Some(command) = receiver.recv().await {
        if let Err(e) = connection_manager.send(command).await {
          error!("Failed to handle gateway command: {:?}", e);
        }
      }
    });

    Ok(())
  }
}
