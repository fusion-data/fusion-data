use std::collections::HashMap;

use fusion_common::time::now_epoch_millis;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  protocol::AgentRegisterRequest,
  types::{CommandKind, EventKind},
};

use super::{AcquireTaskRequest, LogMessage, TaskInstanceChanged};

/// 服务器下发的指令。 Server -> Agent
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct WebSocketCommand {
  /// 指令ID，全局唯一。可选参数，默认使用 UUID v7
  id: Uuid,
  /// 发送时间
  timestamp: i64,
  /// 指令类型
  pub kind: CommandKind,
  /// 指令参数
  pub parameters: serde_json::Value,
  /// 指令超时时间
  pub timeout: Option<u32>,
  /// 指令优先级
  pub priority: Option<u8>, // 指令优先级
}

impl WebSocketCommand {
  pub fn new<T: Serialize>(kind: CommandKind, parameters: T) -> Self {
    Self::new_with_id(Uuid::now_v7(), kind, parameters)
  }

  pub fn new_with_id<T: Serialize>(id: Uuid, kind: CommandKind, parameters: T) -> Self {
    Self {
      id,
      timestamp: now_epoch_millis(),
      kind,
      parameters: serde_json::to_value(parameters).unwrap(),
      timeout: None,
      priority: None,
    }
  }

  pub fn with_timeout(mut self, timeout: u32) -> Self {
    self.timeout = Some(timeout);
    self
  }

  pub fn with_priority(mut self, priority: u8) -> Self {
    self.priority = Some(priority);
    self
  }

  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn timestamp(&self) -> i64 {
    self.timestamp
  }
}

/// WebSocket 事件统一包装器，Agent -> Server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct WebSocketEvent {
  /// 消息唯一标识
  pub event_id: Uuid,
  /// 发送时间
  pub timestamp: i64,
  /// 消息类型
  pub kind: EventKind,
  /// 消息载荷
  pub payload: serde_json::Value,
  /// 扩展元数据
  pub metadata: HashMap<String, String>,
}

impl WebSocketEvent {
  pub fn new_task_log(message: LogMessage) -> Self {
    Self::new_with_id(Uuid::now_v7(), EventKind::TaskLog, message)
  }

  pub fn new_task_instance_updated(message: TaskInstanceChanged) -> Self {
    Self::new_with_id(Uuid::now_v7(), EventKind::TaskInstanceChanged, message)
  }

  pub fn new_poll_task(message: AcquireTaskRequest) -> Self {
    Self::new_with_id(Uuid::now_v7(), EventKind::PollTaskRequest, message)
  }

  pub fn new_agent_register(message: AgentRegisterRequest) -> Self {
    Self::new_with_id(Uuid::now_v7(), EventKind::AgentRegister, message)
  }

  pub fn new_with_id<T: Serialize>(event_id: Uuid, kind: EventKind, payload: T) -> Self {
    Self {
      event_id,
      timestamp: now_epoch_millis(),
      kind,
      payload: serde_json::to_value(payload).unwrap(),
      metadata: HashMap::default(),
    }
  }

  pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
    self.metadata = metadata;
    self
  }

  pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
    self.metadata.insert(key.into(), value.into());
    self
  }
}
