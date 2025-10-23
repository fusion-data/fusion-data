mod agent;
mod distributed_lock;

pub use agent::*;
pub use distributed_lock::*;
use uuid::Uuid;

use std::sync::atomic::{AtomicI64, Ordering};

use fusion_common::time::now_epoch_millis;
use mea::mpsc;
use serde::{Deserialize, Serialize};

use hetuflow_core::protocol::CommandMessage;

use crate::connection::GatewayError;

/// 数据流动方向: Server -> Agent
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CommandMessageRequest {
  Single { command: CommandMessage, agent_id: String },
  Broadcast { command: CommandMessage },
}

impl CommandMessageRequest {
  pub fn command_id(&self) -> &Uuid {
    match self {
      CommandMessageRequest::Single { command, .. } => command.id(),
      CommandMessageRequest::Broadcast { command } => command.id(),
    }
  }
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
  Healthy,
  Unhealthy,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SystemStatus {
  /// 系统状态
  status: HealthStatus,
  /// 数据库连接数量
  db_conn_size: u32,
  /// Agent 在线数量
  agent_size: u32,
  timestamp: i64,
}

impl SystemStatus {
  pub fn new(db_size: u32, agent_size: u32) -> Self {
    Self {
      status: if db_size > 0 && agent_size > 0 { HealthStatus::Healthy } else { HealthStatus::Unhealthy },
      db_conn_size: db_size,
      agent_size,
      timestamp: now_epoch_millis(),
    }
  }
}

#[derive(Serialize)]
pub struct AgentConnInfo {
  /// Agent ID
  pub agent_id: String,
  /// Agent 地址
  pub address: String,
  /// 最后心跳时间（毫秒）
  last_heartbeat_ms: i64,
}

/// Agent 连接信息
pub struct AgentConnection {
  /// Agent ID
  pub agent_id: String,
  /// Agent 地址
  pub address: String,
  /// 最后心跳时间（毫秒）
  last_heartbeat_ms: AtomicI64,
  // 当离线时，sender 为 None
  pub sender: Option<mpsc::UnboundedSender<CommandMessage>>,
}

impl AgentConnection {
  pub fn new(agent_id: String, address: String, sender: mpsc::UnboundedSender<CommandMessage>) -> Self {
    Self { agent_id, address, last_heartbeat_ms: AtomicI64::new(0), sender: Some(sender) }
  }

  pub async fn is_online(&self) -> bool {
    self.sender.is_some()
  }

  /// 发送消息给 Agent
  pub fn send_command(&self, message: CommandMessage) -> Result<(), GatewayError> {
    if let Some(sender) = &self.sender {
      sender
        .send(message)
        .map_err(|e| GatewayError::async_queue_error(format!("Failed to send message: {}", e)))?;
      Ok(())
    } else {
      Err(GatewayError::internal("No WebSocket sender available"))
    }
  }

  pub fn last_heartbeat_ms(&self) -> i64 {
    self.last_heartbeat_ms.load(Ordering::Relaxed)
  }

  pub fn set_last_heartbeat_ms(&self, ms: i64) {
    self.last_heartbeat_ms.store(ms, Ordering::Relaxed);
  }
}

/// 连接统计信息
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStats {
  pub total_agents: usize,
  pub online_agents: usize,
  pub offline_agents: usize,
}
