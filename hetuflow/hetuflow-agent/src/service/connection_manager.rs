use std::sync::{Arc, Mutex};

use fusion_common::{
  ahash::HashMap,
  time::{OffsetDateTime, now_epoch_millis},
};
use fusion_core::DataError;
use futures_util::{FutureExt, pin_mut};
use log::{error, info, warn};
use tokio::{
  sync::{broadcast, mpsc},
  task::JoinHandle,
};

use hetuflow_core::{
  models::AgentMetrics,
  protocol::{HeartbeatRequest, TaskResponse, WebSocketCommand, WebSocketEvent},
  types::{AgentStatus, CommandKind, EventKind, HetuflowCommand},
};

use crate::{service::WsHandler, setting::HetuflowAgentSetting};

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
  command_publisher: broadcast::Sender<HetuflowCommand>,
  shutdown_tx: broadcast::Sender<()>,
  // 在 self.start_websocket 中会将 event_rx 取出来，所以这里需要用 Mutex 保护
  event_rx: Mutex<Option<mpsc::UnboundedReceiver<WebSocketEvent>>>,
  event_tx: mpsc::UnboundedSender<WebSocketEvent>,
  websocket_handle: Mutex<Option<JoinHandle<()>>>,
}

impl ConnectionManager {
  /// 创建新的连接管理器
  pub fn new(setting: Arc<HetuflowAgentSetting>, shutdown_tx: broadcast::Sender<()>) -> Self {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (command_publisher, _) = broadcast::channel(100);
    Self {
      setting,
      command_publisher,
      shutdown_tx,
      event_rx: Mutex::new(Some(event_rx)),
      event_tx,
      websocket_handle: Mutex::new(None),
    }
  }

  pub fn send_event(&self, event: WebSocketEvent) -> Result<(), DataError> {
    self.event_tx.send(event).map_err(DataError::from)
  }

  /// 连接到 Hetuflow Server
  pub async fn start(&self) -> Result<(), DataError> {
    self.start_heartbeat();
    self.start_websocket();

    Ok(())
  }

  pub fn subscribe_command(&self) -> broadcast::Receiver<HetuflowCommand> {
    self.command_publisher.subscribe()
  }

  fn start_websocket(&self) {
    let event_rx = { self.event_rx.lock().unwrap().take().unwrap() };
    let mut ws_handler =
      WsHandler::new(self.setting.clone(), self.command_publisher.clone(), event_rx, self.shutdown_tx.clone());
    let handle = tokio::spawn(async move { ws_handler.start_loop().await });
    let mut websocket_handle = self.websocket_handle.lock().unwrap();
    *websocket_handle = Some(handle);
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

  fn start_heartbeat(&self) {
    let event_tx = self.event_tx.clone();
    let setting = self.setting.clone();
    let mut shutdown_rx = self.shutdown_tx.subscribe();
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(setting.connection.heartbeat_interval);
      loop {
        let shutdown_fut = shutdown_rx.recv().fuse();
        let interval_fut = interval.tick().fuse();
        pin_mut!(shutdown_fut, interval_fut);
        futures_util::select! {
          _ = shutdown_fut => {
            info!("Heartbeat task shutting down");
            break;
          }
          _ = interval_fut => {/* do nothing */},
        }
        let request = HeartbeatRequest {
          agent_id: setting.agent_id,
          timestamp: now_epoch_millis(),
          status: AgentStatus::Online,
          running_tasks: vec![],
          metrics: AgentMetrics::default(),
          last_task_id: None,
        };
        let heartbeat = WebSocketEvent::new(EventKind::AgentHeartbeat, request);
        if let Err(e) = event_tx.send(heartbeat) {
          warn!("Failed to send heartbeat: {}", e);
        }
      }
    });
  }
}
