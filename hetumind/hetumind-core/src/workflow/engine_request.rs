//! Engine Request/Response 数据模型
//!
//! 统一的工具调用中间态结构，供 Agent 节点与引擎/Tool 节点之间传递。
//! - 请求通过 AiTool 端口输出，由引擎路由到对应 Tool 节点执行
//! - 响应由引擎或 Tool 节点返回，携带输出或错误信息

use serde::{Deserialize, Serialize};

use crate::workflow::NodeConnectionKind;

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
  /// 最大重试次数（不含初次执行）
  pub max_retries: u32,
  /// 初始退避时延（毫秒）
  pub initial_backoff_ms: u64,
  /// 退避倍数（指数退避）
  pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
  fn default() -> Self {
    Self { max_retries: 0, initial_backoff_ms: 0, backoff_multiplier: 1.0 }
  }
}

/// 工具调用请求（由 Agent 节点输出到 AiTool 端口）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRequest {
  /// 目标 Tool 节点名称（工作流内唯一）
  pub node_name: String,
  /// 连接类型（固定为 AiTool）
  pub kind: NodeConnectionKind,
  /// 请求唯一 ID（关联响应）
  pub id: String,
  /// 工具输入参数（已按 JSON Schema 校验）
  pub input: serde_json::Value,
  /// 额外元数据（如 itemIndex、上下文信息）
  pub metadata: serde_json::Value,
  /// 关联链路 ID（用于观测与回溯）
  pub correlation_id: Option<String>,
  /// 重试策略（可选）
  pub retry_policy: Option<RetryPolicy>,
}

/// 工具调用响应（由引擎或 Tool 节点返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResponse {
  /// 关联请求 ID
  pub id: String,
  /// 工具输出（JSON）
  pub output: serde_json::Value,
  /// 错误信息（可选）
  pub error: Option<String>,
  /// 关联链路 ID（用于观测与回溯）
  pub correlation_id: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::*;

  /// 测试：EngineRequest/Response 的 serde 序列化与反序列化保持一致
  #[test]
  fn test_engine_request_response_serde_roundtrip() {
    let req = EngineRequest {
      node_name: "tool_wiki".to_string(),
      kind: NodeConnectionKind::AiTool,
      id: "call-1".to_string(),
      input: serde_json::json!({ "query": "Rust" }),
      metadata: serde_json::json!({ "itemIndex": 0 }),
      correlation_id: Some("corr-123".to_string()),
      retry_policy: Some(RetryPolicy { max_retries: 3, initial_backoff_ms: 100, backoff_multiplier: 2.0 }),
    };

    let s = serde_json::to_string(&req).unwrap();
    let back: EngineRequest = serde_json::from_str(&s).unwrap();
    assert_eq!(back.node_name, req.node_name);
    assert_eq!(back.kind, req.kind);
    assert_eq!(back.id, req.id);
    assert_eq!(back.correlation_id, req.correlation_id);

    let resp = EngineResponse {
      id: req.id.clone(),
      output: serde_json::json!({ "summary": "Rust is a systems programming language." }),
      error: None,
      correlation_id: req.correlation_id.clone(),
    };

    let s2 = serde_json::to_string(&resp).unwrap();
    let back2: EngineResponse = serde_json::from_str(&s2).unwrap();
    assert!(back2.error.is_none());
    assert_eq!(back2.correlation_id, resp.correlation_id);
  }
}
