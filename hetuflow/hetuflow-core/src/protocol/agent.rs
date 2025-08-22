use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{AgentCapabilities, AgentEntity};

#[derive(Debug, Deserialize)]
pub struct WebSocketParams {
  pub agent_id: Uuid,
}

/// Agent 注册请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentRegisterRequest {
  /// Agent 唯一标识
  pub agent_id: Uuid,
  /// 命名空间
  pub namespace_id: Uuid,
  /// Agent 能力描述
  pub capabilities: AgentCapabilities,
  /// 扩展元数据
  pub metadata: HashMap<String, String>,
  /// Agent 版本
  pub version: String,
  /// 主机名
  pub hostname: String,
  /// 操作系统信息
  pub os_info: String,
}

/// Agent 注册响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentRegisterResponse {
  /// 注册是否成功
  pub success: bool,
  /// 响应消息
  pub message: String,
  /// Agent 配置
  pub config: Option<AgentEntity>,
  /// 服务器时间
  pub server_time: i64,
  /// 会话标识
  /// - 1. 会话身份验证: Agent 在后续的所有 WebSocket 消息中都需要携带此 session_id ，服务器用它来验证请求的合法性
  /// - 2. 连接状态管理: 服务器通过 session_id 跟踪每个 Agent 的连接状态，实现心跳检测和断线重连
  /// - 3. 消息路由: 在多 Agent 环境中， session_id 帮助服务器准确地将任务分发给正确的 Agent
  pub session_id: String,
}

#[cfg(test)]
mod tests {
  use crate::models::AgentCapabilities;

  use super::*;

  #[test]
  fn test_agent_register_request_creation() {
    let agent_id = Uuid::new_v4();
    let namespace_id = Uuid::new_v4();
    let capabilities = AgentCapabilities {
      tags: HashMap::from([("bash".to_string(), None), ("python".to_string(), None)]),
      max_concurrent_tasks: 5,
      ..Default::default()
    };

    let request = AgentRegisterRequest {
      agent_id,
      namespace_id,
      capabilities: capabilities.clone(),
      metadata: HashMap::from([("region".to_string(), "us-west".to_string())]),
      version: "1.0.0".to_string(),
      hostname: "agent-01".to_string(),
      os_info: "Linux 5.15.0".to_string(),
    };

    assert_eq!(request.agent_id, agent_id);
    assert_eq!(request.namespace_id, namespace_id);
    assert_eq!(request.capabilities.tags.keys().collect::<Vec<_>>(), vec!["bash", "python"]);
    assert_eq!(request.version, "1.0.0");
  }

  #[test]
  fn test_agent_register_request_serialization() {
    let agent_id = Uuid::new_v4();
    let namespace_id = Uuid::new_v4();
    let capabilities = AgentCapabilities {
      tags: HashMap::from([("bash".to_string(), None)]),
      max_concurrent_tasks: 1,
      ..Default::default()
    };

    let request = AgentRegisterRequest {
      agent_id,
      namespace_id,
      capabilities,
      metadata: HashMap::default(),
      version: "1.0.0".to_string(),
      hostname: "test".to_string(),
      os_info: "test".to_string(),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: AgentRegisterRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.agent_id, agent_id);
    assert_eq!(deserialized.version, "1.0.0");
  }

  #[test]
  fn test_agent_register_response_creation() {
    let response = AgentRegisterResponse {
      success: true,
      message: "Registration successful".to_string(),
      config: None,
      server_time: 1234567890,
      session_id: "session-123".to_string(),
    };

    assert!(response.success);
    assert_eq!(response.message, "Registration successful");
    assert_eq!(response.session_id, "session-123");
  }

  #[test]
  fn test_agent_register_response_serialization() {
    let response = AgentRegisterResponse {
      success: false,
      message: "Registration failed".to_string(),
      config: None,
      server_time: 1234567890,
      session_id: "session-456".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: AgentRegisterResponse = serde_json::from_str(&serialized).unwrap();

    assert!(!deserialized.success);
    assert_eq!(deserialized.message, "Registration failed");
    assert_eq!(deserialized.session_id, "session-456");
  }
}
