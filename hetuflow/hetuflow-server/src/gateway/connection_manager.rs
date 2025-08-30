use std::{
  sync::{Arc, RwLock},
  time::Duration,
};

use hetuflow_core::protocol::{HeartbeatRequest, WebSocketCommand};
use log::{debug, error, info};
use tokio::sync::mpsc;
use ultimate_common::{ahash::HashMap, time::now_epoch_millis};
use uuid::Uuid;

use crate::model::{AgentConnection, AgentEvent, ConnectionStats, GatewayCommandRequest};

use super::GatewayError;

/// 连接管理器
pub struct ConnectionManager {
  connections: Arc<RwLock<HashMap<Uuid, Arc<AgentConnection>>>>,
  event_listeners: Arc<RwLock<Vec<mpsc::UnboundedSender<AgentEvent>>>>,
}

impl Default for ConnectionManager {
  fn default() -> Self {
    Self::new()
  }
}

impl ConnectionManager {
  /// 创建新的连接管理器
  pub fn new() -> Self {
    Self { connections: Arc::new(RwLock::new(HashMap::default())), event_listeners: Arc::new(RwLock::new(Vec::new())) }
  }

  /// 添加新连接
  pub fn add_connection(&self, agent_id: Uuid, connection: AgentConnection) -> Result<(), GatewayError> {
    // 添加到内存连接池
    let mut connections = self.connections.write().unwrap();
    connections.insert(agent_id, Arc::new(connection));

    info!("Agent {} connected successfully", agent_id);
    Ok(())
  }

  /// 连接丢失：发送消息失败，但未触发 remove 连接操作
  fn lost_connection(&self, agent_id: Uuid) -> Result<(), GatewayError> {
    // 更新可靠性统计
    let connections = self.connections.read().unwrap();
    if let Some(agent) = connections.get(&agent_id) {
      agent.update_consecutive_failures();
    }
    Ok(())
  }

  /// 移除连接
  pub fn remove_connection(&self, agent_id: Uuid, reason: &str) -> Result<(), GatewayError> {
    let mut connections = self.connections.write().unwrap();
    connections.remove(&agent_id);
    info!("Agent {} disconnected: {}", agent_id, reason);
    Ok(())
  }

  /// 更新心跳时间
  pub fn update_heartbeat(&self, agent_id: &Uuid, timestamp: i64) -> Result<(), GatewayError> {
    let connections = self.connections.write().unwrap();
    if let Some(agent) = connections.get(agent_id) {
      agent.set_last_heartbeat_ms(timestamp);
      // 心跳成功，重置连续失败计数
      agent.reset_consecutive_failures();
    }
    Ok(())
  }

  /// 发送消息
  pub async fn send(&self, command: GatewayCommandRequest) -> Result<(), GatewayError> {
    match command {
      GatewayCommandRequest::Single { agent_id, command } => self.send_to_agent(&agent_id, command),
      GatewayCommandRequest::Broadcast { command } => self.send_to_all(command),
    }
  }

  /// 获取连接统计信息
  pub async fn get_connection_stats(&self) -> Result<ConnectionStats, GatewayError> {
    let connections = self.connections.read().unwrap();
    let total_agents = connections.len();
    let online_agents = connections.iter().filter(|(_, conn)| conn.sender.is_some()).count();
    Ok(ConnectionStats { total_agents, online_agents, offline_agents: total_agents - online_agents })
  }

  /// 清理过期连接
  pub async fn cleanup_stale_connections(&self, timeout: Duration) -> Result<(), GatewayError> {
    let timeout_millis: i64 = timeout.as_millis() as i64;
    let now = now_epoch_millis();

    // 清理内存中的连接
    let mut connections = self.connections.write().unwrap();
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
        self.publish_event(AgentEvent::Unconnected { agent_id, reason: "Heartbeat timeout".to_string() })?;
      }
    }

    if cleaned_count > 0 {
      info!("Cleaned stale connections: {} agents", cleaned_count);
    }

    Ok(())
  }

  /// 发布事件到所有订阅者（内部辅助方法）
  pub fn publish_event(&self, event: AgentEvent) -> Result<(), GatewayError> {
    let senders = self.event_listeners.read().unwrap();
    for tx in senders.iter() {
      let _ = tx.send(event.clone());
    }
    Ok(())
  }

  /// 发送消息给指定 Agent
  pub fn send_to_agent(&self, agent_id: &Uuid, command: WebSocketCommand) -> Result<(), GatewayError> {
    let connections = self.connections.read().unwrap();
    if let Some(connection) = connections.get(agent_id) {
      connection.send_command(command)
    } else {
      Err(GatewayError::connection_not_found(*agent_id))
    }
  }

  /// 广播消息给所有在线 Agent
  fn send_to_all(&self, command: WebSocketCommand) -> Result<(), GatewayError> {
    let connections = self.get_online_agents()?;
    let mut failed_agents = Vec::new();

    for connection in connections {
      if let Err(e) = connection.send_command(command.clone()) {
        error!("Failed to send message to agent {}: {:?}", connection.agent_id, e);
        failed_agents.push(connection.agent_id);
      }
    }

    // 记录失败的 Agent 连接
    for agent_id in failed_agents {
      self.lost_connection(agent_id)?;
    }

    Ok(())
  }

  pub fn get_online_agents(&self) -> Result<Vec<Arc<AgentConnection>>, GatewayError> {
    // 基于内存连接
    let connections = self.connections.read().unwrap();
    let online_agents =
      connections.iter().filter(|(_, conn)| conn.sender.is_some()).map(|(_, conn)| conn.clone()).collect();
    Ok(online_agents)
  }

  pub fn get_agent(&self, agent_id: &Uuid) -> Result<Option<Arc<AgentConnection>>, GatewayError> {
    let connections = self.connections.read().unwrap();
    if let Some(conn) = connections.get(agent_id) { Ok(Some(conn.clone())) } else { Ok(None) }
  }

  pub fn find_online_agent(&self, agent_id: &Uuid) -> Result<Arc<AgentConnection>, GatewayError> {
    if let Some(conn) = self.get_agent(agent_id)?
      && conn.is_online()
    {
      Ok(conn.clone())
    } else {
      Err(GatewayError::connection_not_found(*agent_id))
    }
  }

  pub fn subscribe_event(&self, handler: mpsc::UnboundedSender<AgentEvent>) -> Result<(), GatewayError> {
    let mut senders = self.event_listeners.write().unwrap();
    senders.push(handler);
    Ok(())
  }

  pub fn get_online_count(&self) -> Result<u32, GatewayError> {
    let conns = self.connections.read().unwrap();
    Ok(conns.len() as u32)
  }

  pub fn is_agent_online(&self, agent_id: &Uuid) -> Result<bool, GatewayError> {
    let conns = self.connections.read().unwrap();
    Ok(conns.contains_key(agent_id))
  }
}
