use std::sync::Arc;

use hetuflow_core::protocol::{
  AgentRegisterRequest, AgentRegisterResponse, HeartbeatRequest, TaskInstanceUpdated, TaskPollRequest,
  WebSocketCommand, WebSocketEvent,
};
use hetuflow_core::types::{CommandKind, EventKind};
use log::warn;
use tokio::sync::mpsc;
use ultimate_common::time::now_epoch_millis;
use uuid::Uuid;

use crate::gateway::{AgentEvent, AgentRegistry};
use crate::infra::bmc::TaskInstanceBmc;
use crate::model::AgentConnection;

use super::{ConnectionManager, GatewayError};

/// 消息路由器
pub struct MessageHandler {
  connection_manager: Arc<ConnectionManager>,
  event_sender: mpsc::UnboundedSender<AgentEvent>,
}

impl MessageHandler {
  /// 创建新的消息路由器
  pub fn new(connection_manager: Arc<ConnectionManager>, event_sender: mpsc::UnboundedSender<AgentEvent>) -> Self {
    Self { connection_manager, event_sender }
  }

  /// 处理来自 Agent 的消息
  pub async fn process_message(&self, agent_id: Uuid, event: WebSocketEvent) -> Result<(), GatewayError> {
    match event.kind {
      EventKind::AgentHeartbeat => {
        let request: HeartbeatRequest = serde_json::from_value(event.payload)?;
        self.handle_heartbeat(agent_id, request).await
      }
      EventKind::PollTaskRequest => {
        let request: TaskPollRequest = serde_json::from_value(event.payload)?;
        self
          .event_sender
          .send(AgentEvent::TaskPollRequest { agent_id, request: Box::new(request) })
          .map_err(|e| GatewayError::async_queue_error(e.to_string()))
      }
      EventKind::TaskChangedEvent => {
        let update: TaskInstanceUpdated = serde_json::from_value(event.payload)?;
        self.handle_task_status_update(agent_id, update).await
      }
      EventKind::AgentRegister => {
        let request: AgentRegisterRequest = serde_json::from_value(event.payload)?;
        self.handle_agent_register(agent_id, request).await
      }
      _ => {
        warn!("Unhandled message type: {:?}", event.kind);
        Err(GatewayError::message_routing_failed(format!("Unknown message kind: {:?}", event.kind)))
      }
    }
  }

  /// 处理 Agent 注册
  async fn handle_agent_register(
    &self,
    agent_id: Uuid,
    register_req: AgentRegisterRequest,
  ) -> Result<(), GatewayError> {
    // 发布注册成功事件
    self
      .connection_manager
      .publish_event(AgentEvent::Registered { agent_id, payload: Box::new(register_req) })
      .await?;

    // 发送注册响应
    let response = AgentRegisterResponse {
      success: true,
      message: "Agent registered successfully".to_string(),
      config: None,
      server_time: now_epoch_millis(),
      session_id: Uuid::now_v7().to_string(),
    };

    let message = WebSocketCommand::new(CommandKind::AgentRegistered, serde_json::to_value(response).unwrap());

    self.connection_manager.send_to_agent(&agent_id, message).await
  }

  /// 处理 Agent 心跳
  async fn handle_heartbeat(&self, agent_id: Uuid, request: HeartbeatRequest) -> Result<(), GatewayError> {
    let event = AgentEvent::Heartbeat { agent_id, timestamp: request.timestamp };

    // 更新心跳时间
    self.connection_manager.update_heartbeat(&agent_id, request).await?;

    // 发送心跳事件
    self.event_sender.send(event).map_err(|e| GatewayError::async_queue_error(e.to_string()))?;

    Ok(())
  }

  /// 处理任务状态更新
  async fn handle_task_status_update(&self, agent_id: Uuid, payload: TaskInstanceUpdated) -> Result<(), GatewayError> {
    // TODO: 实现任务状态更新逻辑
    // 1. 解析任务状态更新
    // 2. 验证任务实例
    // 3. 更新任务状态
    // 4. 通知相关服务
    // 发送状态更新事件
    let event = AgentEvent::TaskInstanceChanged { agent_id, payload: Box::new(payload) };
    self.event_sender.send(event).map_err(|e| GatewayError::async_queue_error(e.to_string()))?;

    Ok(())
  }

  pub async fn add_connection(&self, agent_id: Uuid, agent_connection: AgentConnection) -> Result<(), GatewayError> {
    self.connection_manager.add_connection(agent_id, agent_connection).await
  }

  pub async fn remove_connection(&self, agent_id: Uuid, reason: &str) -> Result<(), GatewayError> {
    self.connection_manager.remove_connection(agent_id, reason).await
  }

  pub async fn lost_connection(&self, agent_id: Uuid) -> Result<(), GatewayError> {
    self.connection_manager.lost_connection(agent_id).await
  }
}
