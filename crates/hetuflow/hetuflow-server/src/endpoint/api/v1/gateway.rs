use std::{net::SocketAddr, sync::Arc};

use axum::{
  Json,
  extract::{
    ConnectInfo, Query, State, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  response::Response,
};
use fusion_core::IdUuidResult;
use fusion_web::{WebResult, ok_json};
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use mea::mpsc;

use hetuflow_core::protocol::{EventMessage, WebSocketParams};
use utoipa_axum::router::OpenApiRouter;

use crate::{
  application::ServerApplication,
  connection::MessageHandler,
  model::{AgentConnection, CommandMessageRequest},
};

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(websocket_handler))
    .routes(utoipa_axum::routes!(send_command))
}

/// 发送命令
///
/// 发送网关命令并返回命令 ID 列表
#[utoipa::path(
  post,
  path = "/command",
  request_body = CommandMessageRequest,
  responses(
    (status = 200, description = "命令发送成功", body = IdUuidResult),
    (status = 400, description = "请求参数错误"),
    (status = 500, description = "服务器内部错误")
  ),
  tag = "Gateway"
)]
async fn send_command(
  State(app): State<ServerApplication>,
  Json(command): Json<CommandMessageRequest>,
) -> WebResult<IdUuidResult> {
  let message_id = *command.command_id();
  app.connection_manager.send(command).await?;
  ok_json!(message_id.into())
}

/// Accept WebSocket connection from Agent
#[utoipa::path(
  get,
  path = "/ws",
  params(
    ("agent_id" = String, Query, description = "Agent ID")
  ),
  responses(
    (status = 101, description = "WebSocket connection established")
  ),
  tag = "Gateway"
)]
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

async fn handle_websocket_connection(
  socket: WebSocket,
  address: String,
  agent_id: String,
  message_handler: Arc<MessageHandler>,
) {
  let (mut ws_tx, mut ws_rx) = socket.split();

  // Create an MPSC channel for the current WebSocket connection to send messages from other tasks to this connection
  let (command_tx, mut command_rx) = mpsc::unbounded();
  let agent_connection = AgentConnection::new(agent_id.clone(), address, command_tx);
  if let Err(e) = message_handler.add_connection(&agent_id, agent_connection).await {
    error!("Failed to add agent connection: {:?}", e);
    return;
  }

  // Start an independent task responsible for receiving messages from MPSC (agent_rx) channel and sending them to WebSocket (peer)
  let agent_id2 = agent_id.clone();
  let sender_task = tokio::spawn(async move {
    while let Some(msg) = command_rx.recv().await {
      let msg = serde_json::to_string(&msg).unwrap();
      if let Err(e) = ws_tx.send(Message::Text(msg.into())).await {
        error!("Failed to send message to WebSocket: {:?}", e);
        break;
      }
    }
    info!("WebSocket sender task for agent {} terminated.", agent_id2);
  });

  while let Some(Ok(msg)) = ws_rx.next().await {
    match msg {
      Message::Text(text) => {
        let text_str = text.as_str();
        match serde_json::from_str::<EventMessage>(text_str) {
          Ok(ws_event) => {
            if let Err(e) = message_handler.process_message(agent_id.clone(), ws_event).await {
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
      _ => { /* Ignore other message types */ }
    }
  }

  if let Err(e) = message_handler.remove_connection(&agent_id, "Connection closed").await {
    error!("Failed to remove connection for agent {}: {:?}", agent_id, e);
  }

  //  Abord WebSocket sender loop
  sender_task.abort();
}
