use std::sync::Arc;

use fusion_common::time::now_epoch_millis;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  protocol::{AcquireTaskResponse, AgentRegisterResponse},
  types::CommandKind,
};

pub(crate) trait Command {}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct CommandHead {
  /// Command id, globally unique. Optional parameter, default using UUID v7
  id: Uuid,

  /// Send epoch milliseconds
  timestamp: i64,

  /// Command kind
  kind: CommandKind,
}

/// Command message wrapper, Server -> Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct CommandMessage {
  head: Arc<CommandHead>,

  /// Command payload
  pub payload: Arc<serde_json::Value>,
}

impl CommandMessage {
  pub(crate) fn new<T: Serialize + Command>(kind: CommandKind, payload: T) -> Self {
    Self {
      head: Arc::new(CommandHead { id: Uuid::now_v7(), timestamp: now_epoch_millis(), kind }),
      payload: Arc::new(serde_json::to_value(payload).unwrap()),
    }
  }

  pub fn new_agent_registered(payload: AgentRegisterResponse) -> Self {
    Self::new(CommandKind::AgentRegistered, payload)
  }

  pub fn as_agent_registered(&self) -> Result<AgentRegisterResponse, serde_json::Error> {
    serde_json::from_value(self.payload.as_ref().clone())
  }

  pub fn new_acquire_task(payload: AcquireTaskResponse) -> Self {
    Self::new(CommandKind::TaskAcquired, payload)
  }
  pub fn as_acquire_task(&self) -> Result<AcquireTaskResponse, serde_json::Error> {
    serde_json::from_value(self.payload.as_ref().clone())
  }

  pub fn id(&self) -> &Uuid {
    &self.head.id
  }

  pub fn timestamp(&self) -> i64 {
    self.head.timestamp
  }

  pub fn kind(&self) -> CommandKind {
    self.head.kind
  }
}
