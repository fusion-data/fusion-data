use std::sync::{Arc, Mutex, RwLock};

use futures_util::{FutureExt, SinkExt, StreamExt, pin_mut};
use hetuflow_core::protocol::{TaskPollResponse, WebSocketCommand, WebSocketEvent};
use log::{error, info, warn};
use tokio::{
  sync::{broadcast, mpsc},
  task::JoinHandle,
};
use tokio_tungstenite::tungstenite::Message;
use ultimate_common::time::OffsetDateTime;
use ultimate_core::DataError;

use crate::{
  service::WsHandler,
  setting::{ConnectionConfig, HetuflowAgentSetting},
};

/// WebSocket 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
  /// 断开连接
  Disconnected,
  /// 连接中
  Connecting,
  /// 已连接
  Connected,
  /// 已注册
  Registered,
  /// 连接错误
  Error(String),
}

/// 连接统计信息
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
  /// 连接次数
  pub connection_count: u64,
  /// 重连次数
  pub reconnection_count: u64,
  /// 发送消息数
  pub messages_sent: u64,
  /// 接收消息数
  pub messages_received: u64,
  /// 心跳发送次数
  pub heartbeats_sent: u64,
  /// 最后心跳时间
  pub last_heartbeat_at: Option<OffsetDateTime>,
  /// 连接建立时间
  pub connected_at: Option<OffsetDateTime>,
  /// 最后错误
  pub last_error: Option<String>,
}

/// 连接管理器
/// 负责与 HetuFlow Gateway 的连接管理、心跳机制和消息传输
pub struct ConnectionManager {
  setting: Arc<HetuflowAgentSetting>,
  shutdown_tx: broadcast::Sender<()>,
  event_tx: RwLock<Option<mpsc::UnboundedSender<WebSocketEvent>>>,
  task_poll_resp_tx: mpsc::UnboundedSender<TaskPollResponse>,
  websocket_handle: Mutex<Option<JoinHandle<()>>>,
}

impl ConnectionManager {
  /// 创建新的连接管理器
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    shutdown_tx: broadcast::Sender<()>,
    task_poll_resp_tx: mpsc::UnboundedSender<TaskPollResponse>,
  ) -> Self {
    Self { setting, shutdown_tx, event_tx: RwLock::new(None), task_poll_resp_tx, websocket_handle: Mutex::new(None) }
  }

  pub fn send_event(&self, event: WebSocketEvent) -> Result<(), DataError> {
    let guard = self.event_tx.read().unwrap();
    if let Some(event_tx) = &*guard {
      event_tx.send(event)?;
    }
    Ok(())
  }

  /// 连接到 Hetuflow Server
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Connecting to Hetuflow Server: {}", self.setting.connection.gateway_url());

    self.start_websocket();

    Ok(())
  }

  fn start_websocket(&self) {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    self.set_event_tx(event_tx);
    let mut ws_handler =
      WsHandler::new(self.setting.clone(), self.task_poll_resp_tx.clone(), event_rx, self.shutdown_tx.subscribe());
    let handle = tokio::spawn(async move { ws_handler.start_loop().await });
    let mut websocket_handle = self.websocket_handle.lock().unwrap();
    *websocket_handle = Some(handle);
  }

  fn set_event_tx(&self, event_tx: mpsc::UnboundedSender<WebSocketEvent>) {
    let mut guard = self.event_tx.write().unwrap();
    guard.replace(event_tx);
  }

  pub async fn wait_closed(&self) -> Result<(), DataError> {
    if let Some(websocket_handle) = self.take_websocket_handle()
      && let Err(e) = websocket_handle.await
    {
      error!("Stop websocket receive loop error: {}", e);
    }

    Ok(())
  }

  fn take_websocket_handle(&self) -> Option<JoinHandle<()>> {
    let mut websocket_handle_guard = self.websocket_handle.lock().unwrap();
    websocket_handle_guard.take()
  }
}
