mod agent;
mod distributed_lock;

pub use agent::*;
pub use distributed_lock::*;

use std::sync::{
  Arc, RwLock,
  atomic::{AtomicI64, Ordering},
};

use fusion_common::time::now_epoch_millis;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use hetuflow_core::protocol::WebSocketCommand;

use crate::gateway::GatewayError;

/// 数据流动方向: Server -> Agent
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum GatewayCommandRequest {
  Single { agent_id: Uuid, command: WebSocketCommand },
  Broadcast { command: WebSocketCommand },
}

#[derive(Serialize)]
pub struct HealthStatus {
  /// 系统状态
  status: bool,
  /// 数据库连接数量
  db_size: u32,
  /// Agent 在线数量
  agent_size: u32,
  timestamp: i64,
}

impl HealthStatus {
  pub fn new(db_size: u32, agent_size: u32) -> Self {
    Self { status: db_size > 0 && agent_size > 0, db_size, agent_size, timestamp: now_epoch_millis() }
  }
}

/// Agent 可靠性统计信息
#[derive(Debug, Default, Clone, Serialize)]
pub struct AgentReliabilityStats {
  /// 成功任务数
  pub success_count: u64,
  /// 失败任务数
  pub failure_count: u64,
  /// 总任务数
  pub total_tasks: u64,
  /// 平均响应时间（毫秒）
  pub avg_response_ms: f64,
  /// 最后失败时间（毫秒）
  pub last_failure_ms: i64,
  /// 连续失败次数
  pub consecutive_failures: u32,
}

/// Agent 连接信息
#[derive(Debug, Serialize)]
pub struct AgentConnection {
  /// Agent ID
  pub agent_id: Uuid,
  /// Agent 地址
  pub address: String,
  /// 最后心跳时间（毫秒）
  last_heartbeat_ms: AtomicI64,
  /// 统计信息
  stats: Arc<RwLock<AgentReliabilityStats>>,
  // 当离线时，sender 为 None
  #[serde(skip)]
  pub sender: Option<mpsc::UnboundedSender<WebSocketCommand>>,
}

impl AgentConnection {
  pub fn new(agent_id: Uuid, address: String, sender: mpsc::UnboundedSender<WebSocketCommand>) -> Self {
    Self {
      agent_id,
      address,
      last_heartbeat_ms: AtomicI64::new(0),
      stats: Arc::new(RwLock::new(AgentReliabilityStats::default())),
      sender: Some(sender),
    }
  }

  pub fn is_online(&self) -> bool {
    self.sender.is_some() && self.stats.read().unwrap().consecutive_failures == 0
  }

  /// 发送消息给 Agent
  pub fn send_command(&self, message: WebSocketCommand) -> Result<(), GatewayError> {
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

  pub fn stats(&self) -> AgentReliabilityStats {
    self.stats.read().unwrap().clone()
  }

  pub fn reset_consecutive_failures(&self) {
    let mut stats = self.stats.write().unwrap();
    stats.consecutive_failures = 0;
  }

  pub fn update_consecutive_failures(&self) {
    let mut stats = self.stats.write().unwrap();
    stats.consecutive_failures += 1;
    stats.last_failure_ms = now_epoch_millis();
  }

  pub fn update_stats(&self, success: bool, response_time_ms: f64) {
    let mut stats = self.stats.write().unwrap();
    stats.total_tasks += 1;
    if success {
      stats.success_count += 1;
      stats.consecutive_failures = 0;
    } else {
      stats.failure_count += 1;
      stats.consecutive_failures += 1;
      stats.last_failure_ms = now_epoch_millis();
    }

    // 更新平均响应时间（简单移动平均）
    let total = stats.total_tasks as f64;
    stats.avg_response_ms = (stats.avg_response_ms * (total - 1.0) + response_time_ms) / total;
  }
}

/// 连接统计信息
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStats {
  pub total_agents: usize,
  pub online_agents: usize,
  pub offline_agents: usize,
}

#[cfg(test)]
mod tests {
  use uuid::Uuid;

  use super::*;

  #[test]
  fn test_agent_connection() {
    let agent_id = Uuid::new_v4();
    let address = "127.0.0.1:8080".to_string();
    let (sender, _) = mpsc::unbounded_channel();
    let agent_conn = AgentConnection::new(agent_id, address.clone(), sender);
    let json_text = serde_json::to_string_pretty(&agent_conn).unwrap();
    println!("{}", json_text);

    assert_eq!(agent_conn.agent_id, agent_id);
    assert_eq!(agent_conn.address, address);
  }
}
