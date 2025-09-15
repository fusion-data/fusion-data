use std::sync::Arc;

use fusion_core::{DataError, IdUuidResult};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use hetuflow_core::{
  models::AgentCapabilities,
  protocol::{AcquireTaskResponse, AgentRegisterRequest, WebSocketCommand, WebSocketEvent},
  types::{CommandKind, EventKind, HetuflowCommand},
};
use log::{error, info, warn};
use mea::shutdown::ShutdownRecv;
use tokio::{
  net::TcpStream,
  sync::{broadcast, mpsc},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

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
    let (ws_stream, local_address) = self.connect_with_timeout_and_retry().await?;
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    self.register_agent(&mut ws_tx, local_address).await?;

    loop {
      tokio::select! {
        _ = self.shutdown_rx.is_shutdown() => { // Shutdown signal received
          info!("Shutdown signal received, stopping ConnectionManager loop");
          return Ok(());
        }
        event = self.event_rx.recv() => { // Send event to Server
          if let Some(event) = event {
            let msg = serde_json::to_string(&event).unwrap();
            if let Err(e) = ws_tx.send(Message::Text(msg.into())).await {
              return Err(DataError::server_error(format!("Send message to Server error: {}", e)));
            }
          } else {
            return Err(DataError::server_error("WebSocketEvent channel closed"));
          }
        }
        msg_maybe = ws_rx.next() => { // Receive message from Server
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

  async fn connect_with_timeout_and_retry(
    &self,
  ) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, String), DataError> {
    let url = self.setting.server_gateway_ws();
    let timeout_duration = self.setting.connection.connect_timeout;
    info!("Connecting to Hetuflow Server: {}", url);

    for attempts in 0..self.setting.connection.max_reconnect_attempts {
      let connect_result = tokio::time::timeout(timeout_duration, tokio_tungstenite::connect_async(&url)).await;
      match connect_result {
        Ok(Ok((ws_stream, _response))) => {
          info!("Connected to Hetuflow Server: {}, attempts: {}", url, attempts);
          let tcp_stream = ws_stream.get_ref();
          let local_address = match tcp_stream {
            MaybeTlsStream::Plain(t) => t.local_addr()?.to_string(),
            #[cfg(feature = "native-tls")]
            MaybeTlsStream::NativeTls(t) => t.local_addr()?.to_string(),
            /// Encrypted socket stream using `rustls`.
            #[cfg(feature = "__rustls-tls")]
            MaybeTlsStream::Rustls(t) => t.local_addr()?.to_string(),
            _ => "".to_string(),
          };
          return Ok((ws_stream, local_address));
        }
        Ok(Err(e)) => {
          error!("Failed to connect to gateway: {}", e);
          tokio::time::sleep(self.setting.connection.reconnect_interval).await;
        }
        Err(elapsed) => {
          warn!("WebSocket connection timed out after {} seconds: {}", timeout_duration.as_secs(), elapsed);
          tokio::time::sleep(self.setting.connection.reconnect_interval).await;
        }
      }
    }

    Err(DataError::server_error(format!(
      "Failed to connect to gateway after {} attempts",
      self.setting.connection.max_reconnect_attempts
    )))
  }

  async fn register_agent(
    &self,
    ws_tx: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    address: String,
  ) -> Result<(), DataError> {
    let capabilities = AgentCapabilities {
      max_concurrent_tasks: self.setting.process.max_concurrent_processes,
      labels: self.setting.labels.clone(),
      metadata: self.setting.metadata.clone(),
    };
    let register_req = AgentRegisterRequest {
      agent_id: self.setting.agent_id.clone(),
      capabilities,
      address,
      jwe_token: self.setting.jwe_token.clone(),
    };
    let message = serde_json::to_string(&WebSocketEvent::new(EventKind::AgentRegister, register_req)).unwrap();
    ws_tx
      .send(Message::Text(message.into()))
      .await
      .map_err(|e| DataError::server_error(format!("Send message to Server error: {}", e)))
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
    CommandKind::LogForward => {
      // TODO: 实现日志转发命令处理
      warn!("LogForward command received but not implemented yet");
    }
  }
  Ok(())
}
