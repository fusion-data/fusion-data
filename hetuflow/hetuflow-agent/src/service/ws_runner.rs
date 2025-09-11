use std::sync::Arc;

use fusion_core::{DataError, IdUuidResult};
use futures_util::{FutureExt, SinkExt, StreamExt, pin_mut};
use hetuflow_core::{
  protocol::{AcquireTaskResponse, WebSocketCommand, WebSocketEvent},
  types::{CommandKind, HetuflowCommand},
};
use log::{error, info};
use mea::shutdown::ShutdownRecv;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;

use crate::setting::HetuflowAgentSetting;

pub struct WsRunner {
  setting: Arc<HetuflowAgentSetting>,
  command_publisher: broadcast::Sender<HetuflowCommand>,
  event_rx: mpsc::UnboundedReceiver<WebSocketEvent>,
  shutdown_rx: ShutdownRecv,
}

impl WsRunner {
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    command_publisher: broadcast::Sender<HetuflowCommand>,
    event_rx: mpsc::UnboundedReceiver<WebSocketEvent>,
    shutdown_rx: ShutdownRecv,
  ) -> Self {
    Self { setting, command_publisher, event_rx, shutdown_rx }
  }

  pub async fn run_loop(&mut self) {
    while let Err(e) = self.run_websocket_loop().await
      && !self.shutdown_rx.is_shutdown_now()
    {
      error!("Failed to run websocket loop: {}. Retrying in 10 seconds...", e);

      // 等待 10 秒后重试
      tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }

    // 正常退出
    info!("WsRunner websocket loop closed");
  }

  async fn run_websocket_loop(&mut self) -> Result<(), DataError> {
    let (ws_stream, _resp) = tokio_tungstenite::connect_async(self.setting.server_gateway_ws())
      .await
      .map_err(|e| DataError::server_error(format!("Failed to connect to gateway: {}", e)))?;
    info!("Connected to Hetuflow Server: {}", self.setting.server_gateway_ws());

    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    loop {
      let event_fut = self.event_rx.recv().fuse();
      let ws_rx_fut = ws_rx.next().fuse();
      let shutdown_fut = self.shutdown_rx.is_shutdown().fuse();
      pin_mut!(event_fut, ws_rx_fut, shutdown_fut);
      futures_util::select! {
        _ = shutdown_fut => { // Shutdown signal received
          info!("Shutdown signal received, stopping ConnectionManager loop");
          return Ok(());
        }
        event = event_fut => { // Send event to Server
          if let Some(event) = event {
            let msg = serde_json::to_string(&event).unwrap();
            if let Err(e) = ws_tx.send(Message::Text(msg.into())).await {
              return Err(DataError::server_error(format!("Send message to Server error: {}", e)));
            }
          } else {
            return Err(DataError::server_error("WebSocketEvent channel closed"));
          }
        }
        msg_maybe = ws_rx_fut => { // Receive message from Server
          let msg = if let Some(msg_result) = msg_maybe  {
            match msg_result {
              Ok(msg) => msg,
              Err(e) => {
                return Err(DataError::server_error(format!("WebSocket receive error: {}", e)));
              }
            }
          } else {
            info!("WebSocketMessage channel closed");
            return Ok(());
          };
          match msg {
            Message::Text(text) => {
              if let Ok(cmd) = serde_json::from_str::<WebSocketCommand>(&text)
                && let Err(e) = process_command(&self.command_publisher, cmd)
              {
                return Err(DataError::server_error(format!("Failed to send WebSocket command: {}", e)));
              }
            }
            Message::Binary(bin) => {
              if let Ok(cmd) = serde_json::from_slice::<WebSocketCommand>(&bin)
                && let Err(e) = process_command(&self.command_publisher, cmd)
              {
                return Err(DataError::server_error(format!("Failed to send WebSocket command: {}", e)));
              }
            }
            Message::Close(_) => {
              info!("WebSocketMessage channel closed");
              return Ok(());
            }
            _ => {
              // do nothing
            }
          }
        }
      }
    }
  }
}

/// 处理从 Server 接收到的命令消息
fn process_command(
  command_publisher: &broadcast::Sender<HetuflowCommand>,
  cmd: WebSocketCommand,
) -> Result<(), DataError> {
  match cmd.kind {
    CommandKind::DispatchTask => {
      let resp: AcquireTaskResponse = serde_json::from_value(cmd.parameters).unwrap();
      let _ = command_publisher.send(HetuflowCommand::AcquiredTask(Arc::new(resp)));
    }
    CommandKind::Shutdown => {
      let _ = command_publisher.send(HetuflowCommand::Shutdown);
    }
    CommandKind::UpdateConfig => {
      let _ = command_publisher.send(HetuflowCommand::UpdateConfig);
    }
    CommandKind::ClearCache => {
      let _ = command_publisher.send(HetuflowCommand::ClearCache);
    }
    CommandKind::FetchMetrics => {
      let _ = command_publisher.send(HetuflowCommand::FetchMetrics);
    }
    CommandKind::AgentRegistered => {
      let resp = serde_json::from_value(cmd.parameters).unwrap();
      let _ = command_publisher.send(HetuflowCommand::AgentRegistered(Arc::new(resp)));
    }
    CommandKind::CancelTask => {
      let resp: IdUuidResult = serde_json::from_value(cmd.parameters).unwrap();
      let _ = command_publisher.send(HetuflowCommand::CancelTask(Arc::new(resp.id)));
    }
  }
  Ok(())
}
