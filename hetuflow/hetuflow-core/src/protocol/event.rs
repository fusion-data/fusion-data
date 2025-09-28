use std::sync::Arc;

use fusion_common::time::now_epoch_millis;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{protocol::HeartbeatEvent, types::EventKind};

use super::{AcquireTaskRequest, AgentLogMessage, RegisterAgentRequest, TaskInstanceChanged};

pub(crate) trait Event {}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct EventHead {
  /// Event id
  id: Uuid,
  /// Epoch milliseconds
  timestamp: i64,
  /// Event kind
  kind: EventKind,
}

/// Event message wrapper, Agent -> Server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct EventMessage {
  head: Arc<EventHead>,

  /// Event payload
  payload: Arc<serde_json::Value>,
}

impl EventMessage {
  pub(crate) fn new<T: Serialize + Event>(kind: EventKind, payload: T) -> Self {
    Self {
      head: Arc::new(EventHead { id: Uuid::now_v7(), timestamp: now_epoch_millis(), kind }),
      payload: Arc::new(serde_json::to_value(payload).unwrap()),
    }
  }

  pub fn new_log_message(message: AgentLogMessage) -> Self {
    Self::new(EventKind::LogMessage, message)
  }

  pub fn as_log_message(&self) -> Result<AgentLogMessage, serde_json::Error> {
    serde_json::from_value(self.payload.as_ref().clone())
  }

  pub fn new_task_instance_changed(message: TaskInstanceChanged) -> Self {
    Self::new(EventKind::TaskInstanceChanged, message)
  }

  pub fn as_task_instance_changed(&self) -> Result<TaskInstanceChanged, serde_json::Error> {
    serde_json::from_value(self.payload.as_ref().clone())
  }

  pub fn new_acquire_task(message: AcquireTaskRequest) -> Self {
    Self::new(EventKind::AcquireTask, message)
  }

  pub fn as_acquire_task(&self) -> Result<AcquireTaskRequest, serde_json::Error> {
    serde_json::from_value(self.payload.as_ref().clone())
  }

  pub fn new_register_agent(message: RegisterAgentRequest) -> Self {
    Self::new(EventKind::RegisterAgent, message)
  }

  pub fn as_register_agent(&self) -> Result<RegisterAgentRequest, serde_json::Error> {
    serde_json::from_value(self.payload.as_ref().clone())
  }

  pub fn new_heartbeat(message: HeartbeatEvent) -> Self {
    Self::new(EventKind::Heartbeat, message)
  }

  pub fn as_heartbeat(&self) -> Result<HeartbeatEvent, serde_json::Error> {
    serde_json::from_value(self.payload.as_ref().clone())
  }

  pub fn id(&self) -> &Uuid {
    &self.head.id
  }

  pub fn kind(&self) -> EventKind {
    self.head.kind
  }

  pub fn timestamp(&self) -> i64 {
    self.head.timestamp
  }
}
