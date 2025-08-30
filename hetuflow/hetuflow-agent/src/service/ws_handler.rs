use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use futures_util::{FutureExt, SinkExt, StreamExt, pin_mut};
use hetuflow_core::{
  protocol::{TaskPollResponse, WebSocketCommand, WebSocketEvent},
  types::CommandKind,
};
use log::{error, info};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;
use ultimate_core::DataError;

use crate::setting::HetuflowAgentSetting;

pub struct WsHandler {
  setting: Arc<HetuflowAgentSetting>,
  task_poll_resp_tx: mpsc::UnboundedSender<TaskPollResponse>,
  event_rx: mpsc::UnboundedReceiver<WebSocketEvent>,
  shutdown_tx: broadcast::Sender<()>,
  closed: Arc<AtomicBool>,
}

impl WsHandler {
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    task_poll_resp_tx: mpsc::UnboundedSender<TaskPollResponse>,
    event_rx: mpsc::UnboundedReceiver<WebSocketEvent>,
    shutdown_tx: broadcast::Sender<()>,
  ) -> Self {
    Self { setting, task_poll_resp_tx, event_rx, shutdown_tx, closed: Arc::new(AtomicBool::new(false)) }
  }

  pub async fn start_loop(&mut self) {
    let closed = self.closed.clone();
    let mut shutdown_rx = self.shutdown_tx.subscribe();
    tokio::spawn(async move {
      let _ = shutdown_rx.recv().await;
      closed.store(true, Ordering::Relaxed);
    });

    while let Err(e) = self.start_websocket_loop().await
      && !self.closed.load(Ordering::Relaxed)
    {
      error!("Failed to run websocket loop: {}", e);

      // 等待 10 秒后重试
      tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }

    // 正常退出
    info!("Websocket loop closed");
  }

  async fn start_websocket_loop(&mut self) -> Result<(), DataError> {
    let (ws_stream, _resp) = tokio_tungstenite::connect_async(self.setting.server_gateway_ws())
      .await
      .map_err(|e| DataError::server_error(format!("Failed to connect to gateway: {}", e)))?;
    info!("Connected to Hetuflow Server: {}", self.setting.server_gateway_ws());

    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let mut shutdown_rx = self.shutdown_tx.subscribe();

    loop {
      let event_fut = self.event_rx.recv().fuse();
      let ws_fut = ws_rx.next().fuse();
      let shutdown_fut = shutdown_rx.recv().fuse();
      pin_mut!(event_fut, ws_fut, shutdown_fut);
      futures_util::select! {
        _ = shutdown_fut => { // Shutdown signal received
          info!("Shutdown signal received, stopping ConnectionManager loop");
          self.closed.store(true, Ordering::Relaxed);
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
        msg_maybe = ws_fut => { // Receive message from Server
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
                && let Err(e) = process_command(&self.task_poll_resp_tx, cmd)
              {
                return Err(DataError::server_error(format!("Failed to send WebSocket command: {}", e)));
              }
            }
            Message::Binary(bin) => {
              if let Ok(cmd) = serde_json::from_slice::<WebSocketCommand>(&bin)
                && let Err(e) = process_command(&self.task_poll_resp_tx, cmd)
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
  task_poll_resp_tx: &mpsc::UnboundedSender<TaskPollResponse>,
  cmd: WebSocketCommand,
) -> Result<(), DataError> {
  match cmd.kind {
    CommandKind::DispatchTask => {
      let resp: TaskPollResponse = serde_json::from_value(cmd.parameters).unwrap();
      task_poll_resp_tx.send(resp)?;
    }
    CommandKind::Shutdown => todo!(),
    CommandKind::Restart => todo!(),
    CommandKind::UpdateConfig => todo!(),
    CommandKind::ClearCache => todo!(),
    CommandKind::ReloadTasks => todo!(),
    CommandKind::HealthCheck => todo!(),
    CommandKind::AgentRegistered => todo!(),
    CommandKind::CancelTask => todo!(),
  }
  Ok(())
}
