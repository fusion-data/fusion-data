//! Error trigger configuration parsing utilities

use hetumind_core::workflow::{NodeExecutionError, ParameterMap, ValidationError};
use serde::{Deserialize, Serialize};

/// Error trigger configuration parsed from node parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTriggerParameters {
  /// 错误触发模式
  pub trigger_mode: ErrorTriggerMode,

  /// 监听的工作流ID（为空表示监听所有工作流）
  pub workflow_ids: Vec<String>,

  /// 监听的错误类型
  pub error_types: Vec<ErrorType>,

  /// 监听的节点名称（为空表示监听所有节点）
  pub node_names: Vec<String>,

  /// 错误严重级别过滤
  pub error_severity: Option<ErrorSeverity>,

  /// 是否启用重试
  pub enable_retry: bool,

  /// 最大重试次数
  pub max_retry_count: u32,

  /// 重试间隔（秒）
  pub retry_interval_seconds: u64,

  /// 是否发送通知
  pub send_notification: bool,

  /// 通知方式
  pub notification_methods: Vec<NotificationMethod>,
}

/// 错误触发模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorTriggerMode {
  /// 监听所有工作流错误
  AllWorkflows,
  /// 监听指定工作流错误
  SpecificWorkflows,
  /// 仅当前工作流内部错误
  InternalOnly,
}

/// 错误类型分类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
  /// 节点执行错误
  NodeExecution,
  /// 工作流超时错误
  Timeout,
  /// 资源不足错误
  ResourceExhausted,
  /// 外部服务错误
  ExternalService,
  /// 数据验证错误
  Validation,
  /// 配置错误
  Configuration,
  /// 所有错误类型
  All,
}

/// 错误严重级别
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
  Low,
  Medium,
  High,
  Critical,
}

/// 通知方式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationMethod {
  /// 邮件通知
  Email,
  /// Slack通知
  Slack,
  /// Webhook通知
  Webhook,
  /// 数据库记录
  Database,
}

/// Parse error trigger configuration from node parameters
pub fn parse_error_trigger_parameters(parameters: &ParameterMap) -> Result<ErrorTriggerParameters, NodeExecutionError> {
  let config: ErrorTriggerParameters = parameters.get().map_err(|e| {
    NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "parameters",
      format!("Failed to parse: {}", e),
    ))
  })?;

  // 验证配置
  if config.trigger_mode == ErrorTriggerMode::SpecificWorkflows && config.workflow_ids.is_empty() {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "workflow_ids",
      "At least one workflow ID is required for SpecificWorkflows mode",
    )));
  }

  if config.enable_retry && config.max_retry_count == 0 {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "max_retry_count",
      "Must be greater than 0 when retry is enabled",
    )));
  }

  Ok(config)
}
