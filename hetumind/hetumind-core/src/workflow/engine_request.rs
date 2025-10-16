use ahash::{HashMap, HashMapExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ConnectionKind, ExecutionDataMap, NodeExecutionStatus};
use crate::types::JsonValue;

/// 引擎动作枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineAction {
  /// 执行节点动作
  ExecuteNode(ExecuteNodeAction),
  /// 获取连接数据动作
  GetConnectionData(GetConnectionDataAction),
}

/// 执行节点动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteNodeAction {
  /// 目标节点名称
  pub node_name: String,
  /// 输入数据
  pub input: JsonValue,
  /// 连接类型
  pub connection_type: ConnectionKind,
  /// 动作ID
  pub action_id: Uuid,
  /// 动作元数据
  pub metadata: HashMap<String, JsonValue>,
}

impl ExecuteNodeAction {
  pub fn new(node_name: impl Into<String>, input: JsonValue, connection_type: ConnectionKind, action_id: Uuid) -> Self {
    Self { node_name: node_name.into(), input, connection_type, action_id, metadata: HashMap::default() }
  }

  pub fn with_node_name(mut self, node_name: impl Into<String>) -> Self {
    self.node_name = node_name.into();
    self
  }

  pub fn with_input(mut self, input: JsonValue) -> Self {
    self.input = input;
    self
  }

  pub fn with_connection_type(mut self, connection_type: ConnectionKind) -> Self {
    self.connection_type = connection_type;
    self
  }

  pub fn with_action_id(mut self, action_id: Uuid) -> Self {
    self.action_id = action_id;
    self
  }

  pub fn with_metadata<I, K, V>(mut self, metadata: I) -> Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<JsonValue>,
  {
    self.metadata = metadata.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
    self
  }

  pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
    self.metadata.insert(key.into(), value.into());
    self
  }
}

/// 获取连接数据动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetConnectionDataAction {
  /// 连接类型
  pub connection_type: ConnectionKind,
  /// 连接索引
  pub connection_index: usize,
  /// 动作ID
  pub action_id: Uuid,
  /// 动作元数据
  pub metadata: HashMap<String, JsonValue>,
}

impl GetConnectionDataAction {
  pub fn new(connection_type: ConnectionKind, connection_index: usize, action_id: Uuid) -> Self {
    Self { connection_type, connection_index, action_id, metadata: HashMap::default() }
  }

  pub fn with_connection_type(mut self, connection_type: ConnectionKind) -> Self {
    self.connection_type = connection_type;
    self
  }

  pub fn with_connection_index(mut self, connection_index: usize) -> Self {
    self.connection_index = connection_index;
    self
  }

  pub fn with_action_id(mut self, action_id: Uuid) -> Self {
    self.action_id = action_id;
    self
  }

  pub fn with_metadata<I, K, V>(mut self, metadata: I) -> Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<JsonValue>,
  {
    self.metadata = metadata.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
    self
  }

  pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
    self.metadata.insert(key.into(), value.into());
    self
  }
}

/// 引擎响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResponse<T = HashMap<String, JsonValue>> {
  /// 动作响应列表
  pub action_responses: Vec<EngineResult>,
  /// 响应元数据
  pub metadata: T,
  /// 响应ID（对应请求ID）
  pub response_id: Uuid,
}

/// 引擎结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResult {
  /// 对应的动作
  pub action: EngineAction,
  /// 执行结果数据
  pub data: ExecutionDataMap,
  /// 执行状态
  pub status: NodeExecutionStatus,
  /// 错误信息（如果有）
  pub error: Option<String>,
}

impl EngineResult {
  pub fn new(action: EngineAction, data: ExecutionDataMap, status: NodeExecutionStatus) -> Self {
    Self { action, data, status, error: None }
  }

  pub fn with_action(mut self, action: EngineAction) -> Self {
    self.action = action;
    self
  }

  pub fn with_data(mut self, data: ExecutionDataMap) -> Self {
    self.data = data;
    self
  }

  pub fn with_status(mut self, status: NodeExecutionStatus) -> Self {
    self.status = status;
    self
  }

  pub fn with_error(mut self, error: impl Into<String>) -> Self {
    self.error = Some(error.into());
    self
  }
}

/// 引擎请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRequest<T = HashMap<String, JsonValue>> {
  /// 需要执行的动作列表
  pub actions: Vec<EngineAction>,
  /// 请求元数据
  pub metadata: T,
  /// 请求ID
  pub request_id: Uuid,
}

impl EngineRequest {
  /// 创建新的引擎请求
  pub fn new() -> Self {
    Self { actions: Vec::new(), metadata: HashMap::default(), request_id: Uuid::new_v4() }
  }

  /// 添加执行节点动作
  pub fn add_execute_node_action(
    &mut self,
    node_name: String,
    input: JsonValue,
    connection_type: ConnectionKind,
    metadata: Option<HashMap<String, JsonValue>>,
  ) -> Uuid {
    let action_id = Uuid::new_v4();
    let action = EngineAction::ExecuteNode(ExecuteNodeAction {
      node_name,
      input,
      connection_type,
      action_id,
      metadata: metadata.unwrap_or_default(),
    });
    self.actions.push(action);
    action_id
  }

  /// 添加获取连接数据动作
  pub fn add_get_connection_data_action(
    &mut self,
    connection_type: ConnectionKind,
    connection_index: usize,
    metadata: Option<HashMap<String, JsonValue>>,
  ) -> Uuid {
    let action_id = Uuid::new_v4();
    let action = EngineAction::GetConnectionData(GetConnectionDataAction {
      connection_type,
      connection_index,
      action_id,
      metadata: metadata.unwrap_or_default(),
    });
    self.actions.push(action);
    action_id
  }

  /// 设置元数据
  pub fn set_metadata(&mut self, key: String, value: JsonValue) {
    self.metadata.insert(key, value);
  }
}

impl Default for EngineRequest {
  fn default() -> Self {
    Self::new()
  }
}

impl Default for EngineResponse {
  fn default() -> Self {
    Self::new(Uuid::new_v4())
  }
}

impl EngineResponse {
  /// 创建新的引擎响应
  pub fn new(request_id: Uuid) -> Self {
    Self { action_responses: Vec::new(), metadata: HashMap::new(), response_id: request_id }
  }

  /// 添加动作结果
  pub fn add_action_result(
    &mut self,
    action: EngineAction,
    data: ExecutionDataMap,
    status: NodeExecutionStatus,
    error: Option<String>,
  ) {
    let result = EngineResult { action, data, status, error };
    self.action_responses.push(result);
  }

  /// 设置元数据
  pub fn set_metadata(&mut self, key: String, value: JsonValue) {
    self.metadata.insert(key, value);
  }

  /// 获取成功的结果
  pub fn get_successful_results(&self) -> Vec<&EngineResult> {
    self.action_responses.iter().filter(|r| matches!(r.status, NodeExecutionStatus::Success)).collect()
  }

  /// 获取失败的结果
  pub fn get_failed_results(&self) -> Vec<&EngineResult> {
    self.action_responses.iter().filter(|r| !matches!(r.status, NodeExecutionStatus::Success)).collect()
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_engine_request_creation() {
    let mut request = EngineRequest::new();

    let action_id =
      request.add_execute_node_action("test_node".to_string(), json!({"test": "data"}), ConnectionKind::Main, None);

    assert_eq!(request.actions.len(), 1);
    assert!(request.metadata.is_empty());
    assert_ne!(action_id, Uuid::default());
  }

  #[test]
  fn test_engine_response_creation() {
    let request_id = Uuid::new_v4();
    let response = EngineResponse::new(request_id);

    assert_eq!(response.response_id, request_id);
    assert_eq!(response.action_responses.len(), 0);
  }
}
