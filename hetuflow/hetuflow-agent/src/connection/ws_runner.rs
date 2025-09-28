use std::sync::Arc;

use fusion_core::{
  DataError,
  concurrent::{RetryStrategy, ServiceHandle, ServiceTask},
};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use hetuflow_core::{
  models::AgentCapabilities,
  protocol::{CommandMessage, EventMessage, RegisterAgentRequest},
};
use log::{error, info, warn};
use mea::shutdown::ShutdownRecv;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
  MaybeTlsStream, WebSocketStream,
  tungstenite::{ClientRequestBuilder, Message},
};

use crate::{connection::ConnectionManager, setting::HetuflowAgentSetting};

pub struct WsRunner {
  setting: Arc<HetuflowAgentSetting>,
  connection_manager: Arc<ConnectionManager>,
  shutdown_rx: ShutdownRecv,
  event_rx: mpsc::UnboundedReceiver<EventMessage>,
}

impl ServiceTask<()> for WsRunner {
  fn retry_strategy(&self) -> RetryStrategy {
    RetryStrategy::new_enable().with_retry_limit(360).with_retry_limit(10).with_increase_rate(1.05)
  }

  async fn run_loop(&mut self) -> Result<(), DataError> {
    self.run_websocket_loop().await
  }
}

impl WsRunner {
  pub async fn new(
    setting: Arc<HetuflowAgentSetting>,
    connection_manager: Arc<ConnectionManager>,
    shutdown_rx: ShutdownRecv,
  ) -> Self {
    let (event_sender, event_rx) = mpsc::unbounded_channel();
    connection_manager.set_event_sender(event_sender).await;
    Self { setting, connection_manager, shutdown_rx, event_rx }
  }

  pub fn run(mut self) -> ServiceHandle {
    ServiceHandle::new("WsRunner", tokio::spawn(async move { self.run_loop().await }))
  }

  async fn run_loop(&mut self) {
    while let Err(e) = self.run_websocket_loop().await {
      if self.shutdown_rx.is_shutdown_now() {
        break;
      }

      error!("Failed to run websocket loop: {}. Retrying in 10 seconds...", e);
      tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
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
              if let Ok(cmd) = serde_json::from_str::<CommandMessage>(&text)
                && let Err(e) = self.connection_manager.publish_command(cmd)
              {
                return Err(DataError::server_error(format!("Failed to send WebSocket command: {}", e)));
              }
            }
            Message::Close(_) => {
              info!("WebSocketMessage channel closed");
              return Ok(());
            }
            _ => { /* Ignore other message types */}
          }
        }
      }
    }
  }

  async fn connect_with_timeout_and_retry(
    &self,
  ) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, String), DataError> {
    let url = self.setting.server_gateway_ws();
    let uri = url.as_str().try_into().map_err(|e| DataError::server_error(format!("Invalid url: {}", e)))?;
    let mut crb = ClientRequestBuilder::new(uri);
    if let Some(jwe_token) = self.setting.jwe_token.as_deref() {
      crb = crb.with_header("Authorization", format!("Bearer {}", jwe_token));
    }
    let timeout_duration = self.setting.connection.connect_timeout;
    info!("Connecting to Hetuflow Server: {}", url);

    for attempts in 0..self.setting.connection.max_reconnect_attempts {
      if self.shutdown_rx.is_shutdown_now() {
        return Err(DataError::BizError {
          code: 503,
          msg: "Shutdown signal received, stopped connect to Server".into(),
          detail: None,
        });
      }
      let connect_result = tokio::time::timeout(timeout_duration, tokio_tungstenite::connect_async(crb.clone())).await;
      match connect_result {
        Ok(Ok((ws_stream, _response))) => {
          info!("Successfully connected to Hetuflow Server: {}, attempts: {}", url, attempts);
          let tcp_stream = ws_stream.get_ref();
          let local_address = match tcp_stream {
            MaybeTlsStream::Plain(t) => t.local_addr()?.to_string(),
            // #[cfg(feature = "native-tls")]
            // MaybeTlsStream::NativeTls(t) => t.local_addr()?.to_string(),
            // /// Encrypted socket stream using `rustls`.
            // #[cfg(feature = "__rustls-tls")]
            // MaybeTlsStream::Rustls(t) => t.local_addr()?.to_string(),
            _ => "".to_string(),
          };
          return Ok((ws_stream, local_address));
        }
        Ok(Err(e)) => {
          error!("Failed to connect to gateway: {}, attempts: {}", e, attempts);
          tokio::time::sleep(self.setting.connection.reconnect_interval).await;
        }
        Err(elapsed) => {
          warn!(
            "WebSocket connection timed out after {} seconds: {}, attempts: {}",
            timeout_duration.as_secs(),
            elapsed,
            attempts
          );
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
    let register_req = RegisterAgentRequest {
      agent_id: self.setting.agent_id.clone(),
      capabilities,
      address,
      jwe_token: self.setting.jwe_token.clone(),
    };
    let message = serde_json::to_string(&EventMessage::new_register_agent(register_req)).unwrap();
    ws_tx
      .send(Message::Text(message.into()))
      .await
      .map_err(|e| DataError::server_error(format!("Send message to Server error: {}", e)))
  }
}
