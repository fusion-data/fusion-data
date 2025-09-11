use std::sync::{Arc, Mutex};

use fusion_common::time::OffsetDateTime;
use fusion_core::DataError;
use futures_util::{FutureExt, pin_mut};
use log::{error, info};
use mea::shutdown::ShutdownRecv;
use tokio::{
  sync::{broadcast, mpsc},
  task::JoinHandle,
};

use hetuflow_core::{
  protocol::{AcquireTaskResponse, WebSocketEvent},
  types::HetuflowCommand,
};

use crate::{service::WsRunner, setting::HetuflowAgentSetting};

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
  acquire_task_broadcaster: broadcast::Sender<Arc<AcquireTaskResponse>>,
  command_publisher: broadcast::Sender<HetuflowCommand>,
  event_tx: mpsc::UnboundedSender<WebSocketEvent>,
  ws_runner: Mutex<Option<WsRunner>>,
  command_runner: Mutex<Option<CommandRunner>>,
}

impl ConnectionManager {
  /// 创建新的连接管理器
  pub fn new(setting: Arc<HetuflowAgentSetting>, shutdown_rx: ShutdownRecv) -> Self {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (command_publisher, _) = broadcast::channel(100);
    let (acquire_task_broadcaster, _) = broadcast::channel(100);

    let ws_runner = WsRunner::new(setting.clone(), command_publisher.clone(), event_rx, shutdown_rx.clone());
    let ws_runner = Mutex::new(Some(ws_runner));

    let command_runner = CommandRunner {
      acquire_task_broadcaster: acquire_task_broadcaster.clone(),
      command_rx: command_publisher.subscribe(),
      shutdown_rx,
    };
    let command_runner = Mutex::new(Some(command_runner));

    Self { acquire_task_broadcaster, command_publisher, event_tx, ws_runner, command_runner }
  }

  /// 连接到 Hetuflow Server
  pub fn start(&self) -> Result<Vec<JoinHandle<()>>, DataError> {
    info!("Starting ConnectionManager");
    let h1 = self.spawn_websocket_handler();
    let h2 = self.spawn_command_listener_loop();
    info!("ConnectionManager started");
    Ok(vec![h1, h2])
  }

  fn spawn_websocket_handler(&self) -> JoinHandle<()> {
    let mut ws_handler = self.ws_runner.lock().unwrap();
    if let Some(mut ws_handler) = ws_handler.take() {
      tokio::spawn(async move { ws_handler.run_loop().await })
    } else {
      panic!("ws_handler is None")
    }
  }

  fn spawn_command_listener_loop(&self) -> JoinHandle<()> {
    let mut command_runner = self.command_runner.lock().unwrap();
    if let Some(mut command_runner) = command_runner.take() {
      tokio::spawn(async move { command_runner.run_loop().await })
    } else {
      panic!("command_runner is None")
    }
  }

  pub fn send_event(&self, event: WebSocketEvent) -> Result<(), DataError> {
    self.event_tx.send(event).map_err(DataError::from)
  }

  pub fn subscribe_command(&self) -> broadcast::Receiver<HetuflowCommand> {
    self.command_publisher.subscribe()
  }

  pub fn subscribe_acquire_task(&self) -> broadcast::Receiver<Arc<AcquireTaskResponse>> {
    self.acquire_task_broadcaster.subscribe()
  }
}

struct CommandRunner {
  acquire_task_broadcaster: broadcast::Sender<Arc<AcquireTaskResponse>>,
  command_rx: broadcast::Receiver<HetuflowCommand>,
  shutdown_rx: ShutdownRecv,
}
impl CommandRunner {
  async fn run_loop(&mut self) {
    loop {
      let command_fut = self.command_rx.recv().fuse();
      let shutdown_fut = self.shutdown_rx.is_shutdown().fuse();
      pin_mut!(command_fut, shutdown_fut);

      futures_util::select! {
        command = command_fut => {
          match command {
            Ok(command) => {
              match command {
                HetuflowCommand::AcquiredTask(task_response) => {
                  if let Err(e) = self.acquire_task_broadcaster.send(task_response) {
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
        _ = shutdown_fut => {
          info!("CommandRunner exited.");
          return;
        }
      }
    }
  }
}
