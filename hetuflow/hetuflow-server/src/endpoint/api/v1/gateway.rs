use std::{net::SocketAddr, sync::Arc};

use axum::{
  Json, Router,
  extract::{
    ConnectInfo, Query, State, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  response::Response,
  routing::{any, post},
};
use fusion_core::IdUuidResult;
use fusion_web::{WebResult, ok_json};
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use serde_json::Value;
use tokio::sync::mpsc;
use uuid::Uuid;

use hetuflow_core::protocol::{WebSocketCommand, WebSocketEvent, WebSocketParams};

use crate::{
  application::ServerApplication,
  gateway::MessageHandler,
  model::{AgentConnection, GatewayCommandRequest},
};

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/ws", any(websocket_handler))
    .route("/status", post(get_status))
    .route("/command", post(send_command))
}

async fn get_status(State(app): State<ServerApplication>) -> WebResult<Value> {
  let info = app.agent_stats().await?;
  ok_json!(info)
}

/// 发送命令
///
/// 发送网关命令并返回命令 ID 列表
async fn send_command(
  State(app): State<ServerApplication>,
  Json(command): Json<GatewayCommandRequest>,
) -> WebResult<IdUuidResult> {
  let message_id = app.send_gateway_command(command).await?;
  ok_json!(message_id.into())
}

/// WebSocket 升级处理器
async fn websocket_handler(
  ws: WebSocketUpgrade,
  Query(params): Query<WebSocketParams>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  State(app): State<ServerApplication>,
) -> Response {
  info!("WebSocket connection attempt from: {:?}", addr);
  let address = addr.to_string();
  ws.on_upgrade(move |socket| {
    handle_websocket_connection(socket, address, params.agent_id, app.message_handler.clone())
  })
}

/// 处理 WebSocket 连接
pub async fn handle_websocket_connection(
  socket: WebSocket,
  address: String,
  agent_id: Uuid,
  message_handler: Arc<MessageHandler>,
) {
  let (mut ws_tx, mut ws_rx) = socket.split();

  // 为当前 WebSocket 连接创建一个 MPSC 通道，用于从其他任务发送消息到此连接
  let (command_tx, mut command_rx) = mpsc::unbounded_channel::<WebSocketCommand>();
  let agent_connection = AgentConnection::new(agent_id, address, command_tx);
  if let Err(e) = message_handler.add_connection(agent_id, agent_connection) {
    error!("Failed to add agent connection: {:?}", e);
    return;
  }

  // 启动一个独立的任务，负责从 MPSC（agent_rx） 通道接收消息并发送到 WebSocket（对端）
  let sender_task = tokio::spawn(async move {
    while let Some(msg) = command_rx.recv().await {
      let msg = serde_json::to_string(&msg).unwrap();
      if let Err(e) = ws_tx.send(Message::Text(msg.into())).await {
        error!("Failed to send message to WebSocket: {:?}", e);
        break;
      }
    }
    info!("WebSocket sender task for agent {} terminated.", agent_id);
  });

  // 接收循环
  while let Some(Ok(msg)) = ws_rx.next().await {
    match msg {
      Message::Text(text) => {
        let text_str = text.as_str();
        match serde_json::from_str::<WebSocketEvent>(text_str) {
          Ok(ws_event) => {
            if let Err(e) = message_handler.process_message(agent_id, ws_event).await {
              error!("Failed to process message: {:?}", e);
            }
          }
          Err(e) => {
            error!("Failed to parse WebSocket message: {:?}", e);
          }
        }
      }
      Message::Close(_) => {
        info!("WebSocket connection closed for agent {}", agent_id);
        break;
      }
      _ => {} // 忽略其他消息类型
    }
  }

  // 清理连接
  if let Err(e) = message_handler.remove_connection(agent_id, "Connection closed") {
    error!("Failed to remove connection for agent {}: {:?}", agent_id, e);
  }

  // TODO: 是否有更优雅的处理方式？
  sender_task.abort();
}
