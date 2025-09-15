use std::sync::Arc;

use log::warn;

use hetuflow_core::{protocol::WebSocketEvent, types::EventKind};

use crate::model::{AgentConnection, AgentEvent};

use super::{ConnectionManager, GatewayError};

/// 消息路由器
pub struct MessageHandler {
  connection_manager: Arc<ConnectionManager>,
}

impl MessageHandler {
  /// 创建新的消息路由器
  pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
    Self { connection_manager }
  }

  /// 处理来自 Agent 的消息
  pub async fn process_message(&self, agent_id: String, event: WebSocketEvent) -> Result<(), GatewayError> {
    self.connection_manager.update_heartbeat(&agent_id, event.timestamp)?;
    match event.kind {
      EventKind::AgentHeartbeat => {
        let event = AgentEvent::new_heartbeat(agent_id, serde_json::from_value(event.payload)?);
        self.connection_manager.publish_event(event)?;
      }
      EventKind::PollTaskRequest => {
        let event = AgentEvent::new_task_poll_request(agent_id, serde_json::from_value(event.payload)?);
        self.connection_manager.publish_event(event)?;
      }
      EventKind::TaskChangedEvent => {
        let event = AgentEvent::new_task_instance_changed(agent_id, serde_json::from_value(event.payload)?);
        self.connection_manager.publish_event(event)?;
      }
      EventKind::AgentRegister => {
        let event = AgentEvent::new_register(agent_id, serde_json::from_value(event.payload)?);
        self.connection_manager.publish_event(event)?;
      }
      _ => {
        warn!("Unhandled message: {:?}", event);
      }
    }
    Ok(())
  }

  pub fn add_connection(&self, agent_id: &str, agent_connection: AgentConnection) -> Result<(), GatewayError> {
    let remote_addr = agent_connection.address.clone();
    self.connection_manager.add_connection(agent_id, agent_connection)?;
    self
      .connection_manager
      .publish_event(AgentEvent::Connected { agent_id: agent_id.to_string(), remote_addr })
  }

  pub fn remove_connection(&self, agent_id: &str, reason: &str) -> Result<(), GatewayError> {
    self.connection_manager.remove_connection(agent_id, reason)?;
    self
      .connection_manager
      .publish_event(AgentEvent::Unconnected { agent_id: agent_id.to_string(), reason: reason.to_string() })
  }
}
