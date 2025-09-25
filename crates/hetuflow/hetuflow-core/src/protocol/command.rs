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
  /// 指令ID，全局唯一。可选参数，默认使用 UUID v7
  id: Uuid,

  /// 发送时间
  timestamp: i64,

  /// 指令类型
  kind: CommandKind,
}

/// 服务器下发的指令。 Server -> Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct CommandMessage {
  head: Arc<CommandHead>,

  /// 指令载荷
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
