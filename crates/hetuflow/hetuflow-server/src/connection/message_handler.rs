use std::sync::Arc;

use hetuflow_core::{protocol::EventMessage, types::EventKind};
use log::info;

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
  pub async fn process_message(&self, agent_id: String, event: EventMessage) -> Result<(), GatewayError> {
    self.connection_manager.update_heartbeat(&agent_id, event.timestamp()).await?;
    match event.kind() {
      EventKind::Heartbeat => {
        let event = AgentEvent::new_heartbeat(agent_id, event.as_heartbeat()?);
        self.connection_manager.publish_event(event).await?;
      }
      EventKind::AcquireTask => {
        let event = AgentEvent::new_task_poll_request(agent_id, event.as_acquire_task()?);
        self.connection_manager.publish_event(event).await?;
      }
      EventKind::TaskInstanceChanged => {
        let event = AgentEvent::new_task_instance_changed(agent_id, event.as_task_instance_changed()?);
        self.connection_manager.publish_event(event).await?;
      }
      EventKind::RegisterAgent => {
        let event = AgentEvent::new_register(agent_id, event.as_register_agent()?);
        self.connection_manager.publish_event(event).await?;
      }
      EventKind::LogMessage => {
        let event = AgentEvent::new_task_log(agent_id, event.as_log_message()?);
        self.connection_manager.publish_event(event).await?;
      }
      EventKind::Ack => { /* ignore */ }
      EventKind::Nack => { /* ignore */ }
    }
    Ok(())
  }

  pub async fn add_connection(&self, agent_id: &str, agent_connection: AgentConnection) -> Result<(), GatewayError> {
    let remote_addr = Arc::new(agent_connection.address.clone());
    self.connection_manager.add_connection(agent_id, agent_connection).await?;
    self
      .connection_manager
      .publish_event(AgentEvent::Connected { agent_id: agent_id.to_string(), remote_addr })
      .await
  }

  pub async fn remove_connection(&self, agent_id: &str, reason: impl Into<String>) -> Result<(), GatewayError> {
    let reason: String = reason.into();
    self.connection_manager.remove_connection(agent_id, &reason).await?;
    self
      .connection_manager
      .publish_event(AgentEvent::Unconnected { agent_id: agent_id.to_string(), reason: Arc::new(reason) })
      .await?;
    info!("Agent connection has been closed, agent_id: {}", agent_id);
    Ok(())
  }
}
