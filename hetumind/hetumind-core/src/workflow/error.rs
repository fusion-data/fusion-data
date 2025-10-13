#[cfg(feature = "with-db")]
use fusionsql::SqlError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typed_builder::TypedBuilder;

use crate::{
  types::JsonValue,
  workflow::{ExecutionId, ExecutionMode, NodeDefinitionBuilderError},
};

use super::{ConnectionIndex, ConnectionKind, NodeKind, NodeName, WorkflowId};

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ValidationError {
  #[error("必填字段缺失: {field}")]
  RequiredFieldMissing { field: String },

  #[error("字段值无效 field:{field}, message:{message}")]
  InvalidFieldValue { field: String, message: String, detail: Option<Box<JsonValue>> },

  #[error("连接无效: 从 {src_name}|{src_kind} 到 {dst_name}|{dst_kind}")]
  InvalidConnectionKind { src_name: NodeName, src_kind: ConnectionKind, dst_name: NodeName, dst_kind: ConnectionKind },

  #[error("工作流结构无效: {0}")]
  InvalidWorkflowStructure(String),

  #[error("节点属性验证失败: {0}")]
  NodePropertyValidation(String),

  #[error("输入节点 {src_name}|{src_kind} 未连接")]
  UnconnectedInputPort { src_name: NodeName, src_kind: ConnectionKind },

  #[error("输出节点 {src_name}|{src_kind} -> {dst_name}|{dst_kind} 存在重复连接")]
  DuplicateConnection { src_name: NodeName, src_kind: ConnectionKind, dst_name: NodeName, dst_kind: ConnectionKind },

  #[error("工作流存在循环依赖")]
  WorkflowHasCycles,

  #[error("节点定义未找到: {node_kind}")]
  NodeDefinitionNotFound { node_kind: NodeKind },

  #[error("JSON 解析失败: {0}")]
  JsonParseError(String),
}

impl ValidationError {
  pub fn required_field_missing(field: impl Into<String>) -> Self {
    ValidationError::RequiredFieldMissing { field: field.into() }
  }

  pub fn invalid_field_value(field: impl Into<String>, message: impl Into<String>) -> Self {
    ValidationError::InvalidFieldValue { field: field.into(), message: message.into(), detail: None }
  }

  pub fn invalid_field_value_with_detail<D>(
    field: impl Into<String>,
    message: impl Into<String>,
    detail: Option<D>,
  ) -> Self
  where
    D: Serialize,
  {
    ValidationError::InvalidFieldValue {
      field: field.into(),
      message: message.into(),
      detail: detail.map(|d| Box::new(serde_json::to_value(d).unwrap())),
    }
  }
}

impl From<serde_json::Error> for ValidationError {
  fn from(e: serde_json::Error) -> Self {
    ValidationError::InvalidWorkflowStructure(e.to_string())
  }
}

#[derive(Debug, Error)]
pub enum WorkflowExecutionError {
  #[error("工作流未找到: {workflow_id}")]
  WorkflowNotFound { workflow_id: WorkflowId },

  #[error("执行超时")]
  ExecutionTimeout,

  #[error("执行已取消")]
  ExecutionCancelled,

  #[error("工作流存在循环依赖")]
  CircularDependency,

  #[error("节点执行失败 workflow_id:{workflow_id}, node_name:{node_name}")]
  NodeExecutionFailed { workflow_id: WorkflowId, node_name: NodeName },

  #[error("资源不足")]
  ResourceExhausted,

  #[error("工作流结构无效: {0}")]
  InvalidWorkflowStructure(String),

  #[error("节点执行超时: {node_name}, timeout_seconds: {timeout_seconds}")]
  NodeTimeout { node_name: NodeName, timeout_seconds: u64 },

  #[error("执行限制超过")]
  ExecutionLimitExceeded,

  #[cfg(feature = "with-db")]
  #[error("存储错误: {0}")]
  StoreError(#[from] SqlError),
}

#[derive(Debug, Error)]
pub enum NodeExecutionError {
  #[error("节点参数验证失败, {0}")]
  ParameterValidation(#[from] ValidationError),

  #[error("节点初始化失败: {message}")]
  InitFailed { message: String, cause: Option<Box<dyn std::error::Error + Send + Sync>> },

  #[error("输入数据无效, {connection_kind}|{port_index}")]
  InvalidInputData { connection_kind: ConnectionKind, port_index: ConnectionIndex },

  #[error("节点类型不支持: {node_kind}")]
  UnsupportedNodeKind { node_kind: NodeKind },

  #[error("外部服务错误: {service}")]
  ExternalServiceError { service: String },

  #[error("超时错误")]
  Timeout,

  #[error("节点执行失败: {node_name}, {message:?}")]
  ExecutionFailed { node_name: NodeName, message: Option<String> },

  #[error("当前节点不存在，workflow_id:{workflow_id}, node_name:{node_name}")]
  NodeNotFound { workflow_id: WorkflowId, node_name: NodeName },

  #[error("数据处理错误: {message}")]
  DataProcessingError { message: String },

  #[error("资源不足")]
  ResourceExhausted,

  #[error("输入数据无效: {0}")]
  InvalidInput(String),

  #[error("配置错误: {0}")]
  ConfigurationError(String),

  #[error("连接错误: {0}")]
  ConnectionError(String),
}

#[derive(Debug, Error)]
pub enum TriggerError {
  #[error("触发器启动失败")]
  StartupFailed,

  #[error("触发器配置无效")]
  InvalidConfiguration,

  #[error("Webhook 注册失败")]
  WebhookRegistrationFailed,

  #[error("调度器错误: {message}")]
  SchedulerError { message: String },
}

#[derive(Debug, Error)]
pub enum RegistrationError {
  #[error("节点类型已存在: {node_kind}")]
  NodeKindAlreadyExists { node_kind: NodeKind },

  #[error(transparent)]
  NodeDefinitionBuilderError(#[from] NodeDefinitionBuilderError),
}

// 错误工作流触发数据结构
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct WorkflowErrorData {
  /// 错误来源工作流信息
  pub workflow: WorkflowErrorSource,
  /// 执行错误信息（如果有执行上下文）
  pub execution: Option<ExecutionErrorInfo>,
  /// 触发器错误信息（如果没有执行上下文）
  pub trigger: Option<TriggerErrorInfo>,
}

/// 错误来源工作流信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowErrorSource {
  pub id: WorkflowId,
  pub name: String,
}

/// 执行错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionErrorInfo {
  pub id: ExecutionId,
  pub url: Option<String>,
  pub retry_of: Option<String>,
  pub error: ErrorInfo,
  pub last_node_executed: NodeName,
  pub mode: ExecutionMode,
}

/// 触发器错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerErrorInfo {
  pub error: ErrorInfo,
  pub mode: ExecutionMode,
}

/// 标准化错误信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
  pub message: String,
  pub stack: Option<String>,
  pub name: Option<String>,
  pub description: Option<String>,
  pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}
