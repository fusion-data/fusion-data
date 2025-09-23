use std::sync::Arc;

use fusion_core::DataError;
use mea::rwlock::RwLock;
use tokio::sync::{broadcast, mpsc};

use hetuflow_core::{
  protocol::{AcquireTaskResponse, WebSocketEvent},
  types::HetuflowCommand,
};

/// 连接管理器
/// 负责与 HetuFlow Gateway 的连接管理、心跳机制和消息传输
pub struct ConnectionManager {
  acquire_task_broadcaster: broadcast::Sender<Arc<AcquireTaskResponse>>,
  command_publisher: broadcast::Sender<HetuflowCommand>,
  event_sender: RwLock<Option<mpsc::UnboundedSender<WebSocketEvent>>>,
}

impl Default for ConnectionManager {
  fn default() -> Self {
    Self::new()
  }
}

impl ConnectionManager {
  /// 创建新的连接管理器
  pub fn new() -> Self {
    let (command_publisher, _) = broadcast::channel(100);
    let (acquire_task_broadcaster, _) = broadcast::channel(100);

    Self { acquire_task_broadcaster, command_publisher, event_sender: RwLock::new(None) }
  }

  pub async fn send_event(&self, event: WebSocketEvent) -> Result<(), DataError> {
    let guard = self.event_sender.read().await;
    let event_tx = guard.as_ref().ok_or_else(|| DataError::server_error("Event sender not initialized"))?;
    event_tx.send(event).map_err(DataError::from)
  }

  pub async fn set_event_sender(
    &self,
    event_sender: mpsc::UnboundedSender<WebSocketEvent>,
  ) -> Option<mpsc::UnboundedSender<WebSocketEvent>> {
    let mut guard = self.event_sender.write().await;
    let older = guard.take();
    *guard = Some(event_sender);
    older
  }

  pub async fn clean_event_sender(&self) -> Option<mpsc::UnboundedSender<WebSocketEvent>> {
    let mut guard = self.event_sender.write().await;
    let older = guard.take();
    *guard = None;
    older
  }

  /// Subscribe to commands sent from the Server
  pub fn subscribe_command(&self) -> broadcast::Receiver<HetuflowCommand> {
    self.command_publisher.subscribe()
  }

  pub fn publish_command(&self, command: HetuflowCommand) -> Result<usize, DataError> {
    self
      .command_publisher
      .send(command)
      .map_err(|e| DataError::server_error(format!("Publish command error: {}", e)))
  }

  pub fn subscribe_acquire_task(&self) -> broadcast::Receiver<Arc<AcquireTaskResponse>> {
    self.acquire_task_broadcaster.subscribe()
  }

  pub fn publish_acquire_task(&self, task: Arc<AcquireTaskResponse>) -> Result<usize, DataError> {
    self
      .acquire_task_broadcaster
      .send(task)
      .map_err(|e| DataError::server_error(format!("Publish AcquireTaskResponse error: {}", e)))
  }
}
