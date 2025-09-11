use std::sync::{Arc, Mutex};

use fusion_common::time::OffsetDateTime;
use fusion_core::DataError;
use mea::shutdown::ShutdownRecv;
use tokio::sync::{broadcast, mpsc};

use hetuflow_core::{protocol::WebSocketEvent, types::HetuflowCommand};

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
  event_tx: mpsc::UnboundedSender<WebSocketEvent>,
  ws_handler: Mutex<Option<WsHandler>>,
}

impl ConnectionManager {
  /// 创建新的连接管理器
  pub fn new(setting: Arc<HetuflowAgentSetting>, shutdown_rx: ShutdownRecv) -> Self {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (command_publisher, _) = broadcast::channel(100);
    let ws_handler = WsHandler::new(setting.clone(), command_publisher.clone(), event_rx, shutdown_rx);
    let ws_handler = Mutex::new(Some(ws_handler));

    Self { setting, command_publisher, event_tx, ws_handler }
  }

  /// 连接到 Hetuflow Server
  pub async fn start(&self) -> Result<(), DataError> {
    self.spawn_websocket_handler();
    Ok(())
  }
  fn spawn_websocket_handler(&self) {
    let mut ws_handler = self.ws_handler.lock().unwrap();
    if let Some(mut ws_handler) = ws_handler.take() {
      tokio::spawn(async move { ws_handler.run_loop().await });
    } else {
      panic!("ws_handler is None");
    }
  }

  pub fn send_event(&self, event: WebSocketEvent) -> Result<(), DataError> {
    self.event_tx.send(event).map_err(DataError::from)
  }

  pub fn subscribe_command(&self) -> broadcast::Receiver<HetuflowCommand> {
    self.command_publisher.subscribe()
  }
}
