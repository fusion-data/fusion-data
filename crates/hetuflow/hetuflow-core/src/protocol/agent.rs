use serde::{Deserialize, Serialize};

use crate::models::{AgentCapabilities, SchedAgent};

#[derive(Debug, Deserialize)]
pub struct WebSocketParams {
  pub agent_id: String,
}

/// Agent 注册请求。Agent 连接上 Server 后发送的第一个请求，用于描述当前 Agent 的能力和元数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentRegisterRequest {
  /// Agent 唯一标识
  pub agent_id: String,
  /// Agent 能力描述
  pub capabilities: AgentCapabilities,
  /// Agent 地址
  pub address: String,
  /// JWE Token (用于身份认证)
  pub jwe_token: Option<String>,
}

/// Agent 注册响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentRegisterResponse {
  /// 注册是否成功
  pub success: bool,
  /// 响应消息
  pub message: String,
  /// Agent 配置
  pub agent: Option<SchedAgent>,
  /// 服务器时间
  pub server_time: i64,
  // /// 会话标识
  // /// - 1. 会话身份验证: Agent 在后续的所有 WebSocket 消息中都需要携带此 session_id ，服务器用它来验证请求的合法性
  // /// - 2. 连接状态管理: 服务器通过 session_id 跟踪每个 Agent 的连接状态，实现心跳检测和断线重连
  // /// - 3. 消息路由: 在多 Agent 环境中， session_id 帮助服务器准确地将任务分发给正确的 Agent
  // pub session_id: String,
}
