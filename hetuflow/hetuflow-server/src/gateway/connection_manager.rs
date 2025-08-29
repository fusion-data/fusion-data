use std::{sync::Arc, time::Duration};

use hetuflow_core::protocol::{GatewayCommand, HeartbeatRequest, WebSocketCommand};
use log::{debug, error, info};
use tokio::sync::{RwLock, mpsc};
use ultimate_common::{ahash::HashMap, time::now_epoch_millis};
use uuid::Uuid;

use crate::model::{AgentConnection, ConnectionStats};

use super::{AgentEvent, AgentRegistry, GatewayError};

/// 连接管理器
pub struct ConnectionManager {
  connections: Arc<RwLock<HashMap<Uuid, Arc<AgentConnection>>>>,
  event_senders: Arc<RwLock<Vec<mpsc::UnboundedSender<AgentEvent>>>>,
}

impl ConnectionManager {
  #[allow(clippy::new_without_default)]
  /// 创建新的连接管理器
  pub fn new() -> Self {
    Self { connections: Arc::new(RwLock::new(HashMap::default())), event_senders: Arc::new(RwLock::new(Vec::new())) }
  }

  /// 添加新连接
  pub async fn add_connection(&self, agent_id: Uuid, connection: AgentConnection) -> Result<(), GatewayError> {
    info!("Agent {} connected successfully", agent_id);

    let remote_addr = connection.address.clone();
    // 添加到内存连接池
    {
      let mut connections = self.connections.write().await;
      connections.insert(agent_id, Arc::new(connection));
    }
    info!("Agent {} connected and registered", agent_id);

    self.publish_event(AgentEvent::Connected { agent_id, remote_addr }).await
  }

  /// 丢失连接
  pub async fn lost_connection(&self, agent_id: Uuid) -> Result<(), GatewayError> {
    // 更新可靠性统计
    let connections = self.connections.read().await;
    if let Some(agent) = connections.get(&agent_id) {
      agent.update_consecutive_failures();
    }
    Ok(())
  }

  /// 移除连接
  pub async fn remove_connection(&self, agent_id: Uuid, reason: &str) -> Result<(), GatewayError> {
    // 记录断开连接日志
    info!("Agent {} disconnected: {}", agent_id, reason);

    // 从内存连接池移除
    {
      let mut connections = self.connections.write().await;
      connections.remove(&agent_id);
    }
    info!("Agent {} disconnected: {}", agent_id, reason);

    // 发布 Disconnected 事件
    self.publish_event(AgentEvent::Unregistered { agent_id, reason: reason.to_string() }).await
  }

  /// 更新心跳时间
  pub async fn update_heartbeat(&self, agent_id: &Uuid, request: HeartbeatRequest) -> Result<(), GatewayError> {
    {
      let connections = self.connections.write().await;
      if let Some(agent) = connections.get(agent_id) {
        agent.set_last_heartbeat_ms(request.timestamp);
        // 心跳成功，重置连续失败计数
        agent.reset_consecutive_failures();
      }
    }

    // 发布 Heartbeat 事件
    let event = AgentEvent::Heartbeat { agent_id: *agent_id, timestamp: request.timestamp };
    self.publish_event(event).await
  }

  /// 发送消息
  pub async fn send(&self, command: GatewayCommand) -> Result<(), GatewayError> {
    match command {
      GatewayCommand::Send { agent_id, command } => self.send_to_agent(&agent_id, command).await,
      GatewayCommand::Broadcast { command } => self.broadcast_to_all(command).await,
    }
  }

  /// 获取连接统计信息
  pub async fn get_connection_stats(&self) -> Result<ConnectionStats, GatewayError> {
    let connections = self.connections.read().await;
    let total_agents = connections.len();
    let online_agents = connections.iter().filter(|(_, conn)| conn.sender.is_some()).count();
    Ok(ConnectionStats { total_agents, online_agents, offline_agents: total_agents - online_agents })
  }

  /// 清理过期连接
  pub async fn cleanup_stale_connections(&self, timeout: Duration) -> Result<(), GatewayError> {
    let timeout_millis: i64 = timeout.as_millis() as i64;
    let now = now_epoch_millis();

    // 清理内存中的连接
    let mut connections = self.connections.write().await;
    let agent_ids: Vec<Uuid> = connections.keys().cloned().collect();
    let mut cleaned_count = 0;

    for agent_id in agent_ids {
      let last_heartbeat_ms = if let Some(conn) = connections.get(&agent_id) { conn.last_heartbeat_ms() } else { 0 };
      if last_heartbeat_ms == 0 || now - last_heartbeat_ms < timeout_millis {
        continue;
      }

      debug!("Agent {} heartbeat timeout, last heartbeat: {}", agent_id, last_heartbeat_ms);
      if connections.remove(&agent_id).is_some() {
        cleaned_count += 1;
        info!("Cleaned stale connection for agent {}, last heartbeat: {}", agent_id, last_heartbeat_ms);

        // 发布断连事件
        self
          .publish_event(AgentEvent::Unregistered { agent_id, reason: "Heartbeat timeout".to_string() })
          .await?;
      }
    }

    if cleaned_count > 0 {
      info!("Cleaned stale connections: {} agents", cleaned_count);
    }

    Ok(())
  }

  /// 发布事件到所有订阅者（内部辅助方法）
  pub async fn publish_event(&self, event: AgentEvent) -> Result<(), GatewayError> {
    let senders = self.event_senders.read().await;
    for tx in senders.iter() {
      let _ = tx.send(event.clone());
    }
    Ok(())
  }
}

#[async_trait::async_trait]
impl AgentRegistry for ConnectionManager {
  /// 发送消息给指定 Agent
  async fn send_to_agent(&self, agent_id: &Uuid, command: WebSocketCommand) -> Result<(), GatewayError> {
    let connections = self.connections.read().await;
    if let Some(connection) = connections.get(agent_id) {
      connection.send_command(command).await
    } else {
      Err(GatewayError::connection_not_found(*agent_id))
    }
  }

  /// 广播消息给所有在线 Agent
  async fn broadcast_to_all(&self, command: WebSocketCommand) -> Result<(), GatewayError> {
    let connections = self.get_online_agents().await?;
    let mut failed_agents = Vec::new();

    for connection in connections {
      if let Err(e) = connection.send_command(command.clone()).await {
        error!("Failed to send message to agent {}: {:?}", connection.agent_id, e);
        failed_agents.push(connection.agent_id);
      }
    }

    // 记录失败的 Agent 连接
    for agent_id in failed_agents {
      self.lost_connection(agent_id).await?;
    }

    Ok(())
  }

  async fn get_online_agents(&self) -> Result<Vec<Arc<AgentConnection>>, GatewayError> {
    // 基于内存连接
    let connections = self.connections.read().await;
    let online_agents =
      connections.iter().filter(|(_, conn)| conn.sender.is_some()).map(|(_, conn)| conn.clone()).collect();
    Ok(online_agents)
  }

  async fn get_agent(&self, agent_id: &Uuid) -> Result<Option<Arc<AgentConnection>>, GatewayError> {
    let connections = self.connections.read().await;
    if let Some(conn) = connections.get(agent_id) { Ok(Some(conn.clone())) } else { Ok(None) }
  }

  async fn find_online_agent(&self, agent_id: &Uuid) -> Result<Arc<AgentConnection>, GatewayError> {
    if let Some(conn) = self.get_agent(agent_id).await?
      && conn.is_online()
    {
      Ok(conn.clone())
    } else {
      Err(GatewayError::connection_not_found(*agent_id))
    }
  }

  async fn subscribe_events(&self, handler: mpsc::UnboundedSender<AgentEvent>) -> Result<(), GatewayError> {
    let mut senders = self.event_senders.write().await;
    senders.push(handler);
    Ok(())
  }

  async fn get_online_count(&self) -> Result<u32, GatewayError> {
    let conns = self.connections.read().await;
    Ok(conns.len() as u32)
  }

  async fn is_agent_online(&self, agent_id: &Uuid) -> Result<bool, GatewayError> {
    let conns = self.connections.read().await;
    Ok(conns.contains_key(agent_id))
  }
}
