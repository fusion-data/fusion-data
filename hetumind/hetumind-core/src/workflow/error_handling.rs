//! 分层错误处理机制
//!
//! 实现工作流执行过程中的错误分类、处理策略和恢复机制

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::{ExecutionId, NodeName, WorkflowId};

/// 工作流错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowError {
  /// 节点执行错误
  NodeExecution(NodeExecutionFailure),
  /// 资源错误
  Resource(ResourceError),
  /// 数据流错误
  DataFlow(DataFlowError),
  /// 系统错误
  System(SystemError),
  /// 业务逻辑错误
  Business(BusinessError),
  /// 配置错误
  Configuration(ConfigurationError),
}

/// 节点执行错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionFailure {
  /// 错误ID
  pub error_id: Uuid,
  /// 执行ID
  pub execution_id: ExecutionId,
  /// 工作流ID
  pub workflow_id: WorkflowId,
  /// 节点名称
  pub node_name: NodeName,
  /// 错误类型
  pub error_type: NodeErrorType,
  /// 错误代码
  pub error_code: String,
  /// 错误消息
  pub error_message: String,
  /// 错误详情
  pub details: HashMap<String, serde_json::Value>,
  /// 发生时间
  pub occurred_at: DateTime<Utc>,
  /// 重试次数
  pub retry_count: u32,
  /// 是否可重试
  pub retryable: bool,
  /// 严重级别
  pub severity: ErrorSeverity,
}

/// 节点错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeErrorType {
  /// 配置错误
  Configuration,
  /// 输入验证错误
  InputValidation,
  /// 执行超时
  Timeout,
  /// 资源不足
  ResourceExhaustion,
  /// 外部服务错误
  ExternalService,
  /// 数据处理错误
  DataProcessing,
  /// 逻辑错误
  Logic,
  /// 权限错误
  Permission,
  /// 网络错误
  Network,
  /// 未知错误
  Unknown,
}

/// 资源错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceError {
  /// 错误ID
  pub error_id: Uuid,
  /// 执行ID
  pub execution_id: ExecutionId,
  /// 资源类型
  pub resource_type: String,
  /// 错误代码
  pub error_code: String,
  /// 错误消息
  pub error_message: String,
  /// 发生时间
  pub occurred_at: DateTime<Utc>,
  /// 重试次数
  pub retry_count: u32,
  /// 严重级别
  pub severity: ErrorSeverity,
}

/// 数据流错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowError {
  /// 错误ID
  pub error_id: Uuid,
  /// 执行ID
  pub execution_id: ExecutionId,
  /// 源节点
  pub source_node: NodeName,
  /// 目标节点
  pub target_node: NodeName,
  /// 连接类型
  pub connection_type: String,
  /// 错误代码
  pub error_code: String,
  /// 错误消息
  pub error_message: String,
  /// 发生时间
  pub occurred_at: DateTime<Utc>,
  /// 严重级别
  pub severity: ErrorSeverity,
}

/// 系统错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemError {
  /// 错误ID
  pub error_id: Uuid,
  /// 执行ID
  pub execution_id: ExecutionId,
  /// 组件名称
  pub component: String,
  /// 错误代码
  pub error_code: String,
  /// 错误消息
  pub error_message: String,
  /// 堆栈跟踪
  pub stack_trace: Option<String>,
  /// 发生时间
  pub occurred_at: DateTime<Utc>,
  /// 严重级别
  pub severity: ErrorSeverity,
}

/// 业务逻辑错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessError {
  /// 错误ID
  pub error_id: Uuid,
  /// 执行ID
  pub execution_id: ExecutionId,
  /// 业务规则
  pub business_rule: String,
  /// 错误代码
  pub error_code: String,
  /// 错误消息
  pub error_message: String,
  /// 业务数据
  pub business_data: HashMap<String, serde_json::Value>,
  /// 发生时间
  pub occurred_at: DateTime<Utc>,
  /// 严重级别
  pub severity: ErrorSeverity,
}

/// 配置错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationError {
  /// 错误ID
  pub error_id: Uuid,
  /// 配置路径
  pub config_path: String,
  /// 错误代码
  pub error_code: String,
  /// 错误消息
  pub error_message: String,
  /// 当前值
  pub current_value: Option<serde_json::Value>,
  /// 期望值
  pub expected_value: Option<serde_json::Value>,
  /// 发生时间
  pub occurred_at: DateTime<Utc>,
  /// 严重级别
  pub severity: ErrorSeverity,
}

/// 错误严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
  /// 调试信息
  Debug,
  /// 信息
  Info,
  /// 警告
  Warning,
  /// 错误
  Error,
  /// 严重错误
  Critical,
  /// 致命错误
  Fatal,
}

/// 错误处理策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowErrorHandlingStrategy {
  /// 忽略错误
  Ignore,
  /// 记录日志
  Log,
  /// 重试
  Retry {
    /// 最大重试次数
    max_retries: u32,
    /// 重试间隔（毫秒）
    retry_interval_ms: u64,
    /// 退避策略
    backoff_strategy: BackoffStrategy,
  },
  /// 回滚
  Rollback,
  /// 跳过节点
  SkipNode,
  /// 使用默认值
  UseDefault,
  /// 路转到错误处理节点
  RouteToErrorHandler {
    /// 错误处理节点名称
    error_handler_node: NodeName,
  },
  /// 终止工作流
  TerminateWorkflow,
  /// 暂停工作流
  PauseWorkflow,
}

/// 退避策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
  /// 固定间隔
  Fixed,
  /// 线性退避
  Linear,
  /// 指数退避
  Exponential {
    /// 基础延迟（毫秒）
    base_delay_ms: u64,
    /// 最大延迟（毫秒）
    max_delay_ms: u64,
    /// 退避倍数
    multiplier: f64,
  },
}

/// 错误处理规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingRule {
  /// 规则ID
  pub rule_id: Uuid,
  /// 规则名称
  pub name: String,
  /// 错误类型过滤器
  pub error_type_filter: Option<String>,
  /// 错误代码过滤器
  pub error_code_filter: Option<String>,
  /// 节点过滤器
  pub node_filter: Option<String>,
  /// 严重级别过滤器
  pub severity_filter: Option<ErrorSeverity>,
  /// 处理策略
  pub strategy: WorkflowErrorHandlingStrategy,
  /// 是否启用
  pub enabled: bool,
  /// 优先级
  pub priority: u32,
  /// 创建时间
  pub created_at: DateTime<Utc>,
}

/// 错误处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingResult {
  /// 已忽略
  Ignored,
  /// 已记录日志
  Logged,
  /// 将重试
  WillRetry {
    /// 重试次数
    retry_count: u32,
    /// 下次重试时间
    next_retry_at: DateTime<Utc>,
  },
  /// 已回滚
  RolledBack,
  /// 节点已跳过
  NodeSkipped,
  /// 已使用默认值
  DefaultUsed,
  /// 已路由到错误处理节点
  RoutedToErrorHandler { error_handler_node: NodeName },
  /// 工作流已终止
  WorkflowTerminated,
  /// 工作流已暂停
  WorkflowPaused,
  /// 处理失败
  Failed { reason: String },
}

/// 错误处理器
#[derive(Debug)]
pub struct ErrorHandler {
  /// 错误处理规则
  rules: Arc<tokio::sync::RwLock<Vec<ErrorHandlingRule>>>,
  /// 错误统计
  stats: Arc<tokio::sync::Mutex<ErrorStats>>,
}

/// 错误统计
#[derive(Debug, Default, Clone)]
pub struct ErrorStats {
  /// 总错误数
  pub total_errors: u64,
  /// 按类型分类的错误数
  pub errors_by_type: HashMap<String, u64>,
  /// 按严重级别分类的错误数
  pub errors_by_severity: HashMap<ErrorSeverity, u64>,
  /// 处理成功数
  pub handled_successfully: u64,
  /// 处理失败数
  pub handling_failed: u64,
}

impl ErrorHandler {
  /// 创建新的错误处理器
  pub fn new() -> Self {
    Self {
      rules: Arc::new(tokio::sync::RwLock::new(Vec::new())),
      stats: Arc::new(tokio::sync::Mutex::new(ErrorStats::default())),
    }
  }

  /// 添加错误处理规则
  pub async fn add_rule(&self, rule: ErrorHandlingRule) {
    let mut rules = self.rules.write().await;
    rules.push(rule);
    // 按优先级排序
    rules.sort_by(|a, b| b.priority.cmp(&a.priority));
  }

  /// 移除错误处理规则
  pub async fn remove_rule(&self, rule_id: Uuid) -> bool {
    let mut rules = self.rules.write().await;
    if let Some(pos) = rules.iter().position(|r| r.rule_id == rule_id) {
      rules.remove(pos);
      true
    } else {
      false
    }
  }

  /// 处理错误
  pub async fn handle_error(&self, error: WorkflowError) -> ErrorHandlingResult {
    // 更新统计
    self.update_stats(&error).await;

    // 查找匹配的规则
    let rules = self.rules.read().await;
    for rule in rules.iter() {
      if rule.enabled && self.matches_rule(&error, rule) {
        return self.apply_strategy(&error, &rule.strategy).await;
      }
    }

    // 默认处理策略：记录日志并终止工作流
    log::error!("Unhandled workflow error: {:?}", error);
    ErrorHandlingResult::WorkflowTerminated
  }

  /// 检查错误是否匹配规则
  fn matches_rule(&self, error: &WorkflowError, rule: &ErrorHandlingRule) -> bool {
    // 检查错误类型
    if let Some(error_type_filter) = &rule.error_type_filter {
      let error_type = match error {
        WorkflowError::NodeExecution(_) => "NodeExecution",
        WorkflowError::Resource(_) => "Resource",
        WorkflowError::DataFlow(_) => "DataFlow",
        WorkflowError::System(_) => "System",
        WorkflowError::Business(_) => "Business",
        WorkflowError::Configuration(_) => "Configuration",
      };
      if error_type != error_type_filter {
        return false;
      }
    }

    // 检查错误代码
    if let Some(error_code_filter) = &rule.error_code_filter {
      let error_code = match error {
        WorkflowError::NodeExecution(e) => &e.error_code,
        WorkflowError::Resource(e) => &e.error_code,
        WorkflowError::DataFlow(e) => &e.error_code,
        WorkflowError::System(e) => &e.error_code,
        WorkflowError::Business(e) => &e.error_code,
        WorkflowError::Configuration(e) => &e.error_code,
      };
      if error_code != error_code_filter {
        return false;
      }
    }

    // 检查节点过滤器
    if let Some(node_filter) = &rule.node_filter {
      let node_name = match error {
        WorkflowError::NodeExecution(e) => &e.node_name,
        WorkflowError::DataFlow(e) => &e.target_node,
        _ => return false,
      };
      if !node_name.as_ref().contains(node_filter) {
        return false;
      }
    }

    // 检查严重级别
    if let Some(severity_filter) = &rule.severity_filter {
      let severity = match error {
        WorkflowError::NodeExecution(e) => e.severity,
        WorkflowError::Resource(e) => e.severity,
        WorkflowError::DataFlow(e) => e.severity,
        WorkflowError::System(e) => e.severity,
        WorkflowError::Business(e) => e.severity,
        WorkflowError::Configuration(e) => e.severity,
      };
      if severity != *severity_filter {
        return false;
      }
    }

    true
  }

  /// 应用处理策略
  async fn apply_strategy(
    &self,
    error: &WorkflowError,
    strategy: &WorkflowErrorHandlingStrategy,
  ) -> ErrorHandlingResult {
    match strategy {
      WorkflowErrorHandlingStrategy::Ignore => {
        log::warn!("Ignoring error: {:?}", error);
        ErrorHandlingResult::Ignored
      }
      WorkflowErrorHandlingStrategy::Log => {
        log::error!("Logged error: {:?}", error);
        ErrorHandlingResult::Logged
      }
      WorkflowErrorHandlingStrategy::Retry { max_retries, retry_interval_ms, backoff_strategy } => {
        let retry_count = match error {
          WorkflowError::NodeExecution(e) => e.retry_count,
          _ => 0,
        };

        if retry_count >= *max_retries {
          log::error!("Max retries exceeded for error: {:?}", error);
          ErrorHandlingResult::Failed { reason: "Max retries exceeded".to_string() }
        } else {
          let delay = self.calculate_retry_delay(retry_count, *retry_interval_ms, backoff_strategy);
          let next_retry_at = Utc::now() + chrono::Duration::milliseconds(delay as i64);

          log::info!("Scheduling retry {} for error at: {:?}", retry_count + 1, next_retry_at);
          ErrorHandlingResult::WillRetry { retry_count: retry_count + 1, next_retry_at }
        }
      }
      WorkflowErrorHandlingStrategy::Rollback => {
        log::info!("Rolling back due to error: {:?}", error);
        ErrorHandlingResult::RolledBack
      }
      WorkflowErrorHandlingStrategy::SkipNode => {
        log::info!("Skipping node due to error: {:?}", error);
        ErrorHandlingResult::NodeSkipped
      }
      WorkflowErrorHandlingStrategy::UseDefault => {
        log::info!("Using default value due to error: {:?}", error);
        ErrorHandlingResult::DefaultUsed
      }
      WorkflowErrorHandlingStrategy::RouteToErrorHandler { error_handler_node } => {
        log::info!("Routing to error handler node: {}", error_handler_node);
        ErrorHandlingResult::RoutedToErrorHandler { error_handler_node: error_handler_node.clone() }
      }
      WorkflowErrorHandlingStrategy::TerminateWorkflow => {
        log::error!("Terminating workflow due to error: {:?}", error);
        ErrorHandlingResult::WorkflowTerminated
      }
      WorkflowErrorHandlingStrategy::PauseWorkflow => {
        log::warn!("Pausing workflow due to error: {:?}", error);
        ErrorHandlingResult::WorkflowPaused
      }
    }
  }

  /// 计算重试延迟
  fn calculate_retry_delay(&self, retry_count: u32, base_interval_ms: u64, strategy: &BackoffStrategy) -> u64 {
    match strategy {
      BackoffStrategy::Fixed => base_interval_ms,
      BackoffStrategy::Linear => base_interval_ms * (retry_count + 1) as u64,
      BackoffStrategy::Exponential { base_delay_ms, max_delay_ms, multiplier } => {
        let delay = (*base_delay_ms as f64) * multiplier.powi(retry_count as i32);
        delay.min(*max_delay_ms as f64) as u64
      }
    }
  }

  /// 更新统计信息
  async fn update_stats(&self, error: &WorkflowError) {
    let mut stats = self.stats.lock().await;
    stats.total_errors += 1;

    let error_type = match error {
      WorkflowError::NodeExecution(_) => "NodeExecution",
      WorkflowError::Resource(_) => "Resource",
      WorkflowError::DataFlow(_) => "DataFlow",
      WorkflowError::System(_) => "System",
      WorkflowError::Business(_) => "Business",
      WorkflowError::Configuration(_) => "Configuration",
    };

    *stats.errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;

    let severity = match error {
      WorkflowError::NodeExecution(e) => e.severity,
      WorkflowError::Resource(e) => e.severity,
      WorkflowError::DataFlow(e) => e.severity,
      WorkflowError::System(e) => e.severity,
      WorkflowError::Business(e) => e.severity,
      WorkflowError::Configuration(e) => e.severity,
    };

    *stats.errors_by_severity.entry(severity).or_insert(0) += 1;
  }

  /// 获取统计信息
  pub async fn get_stats(&self) -> ErrorStats {
    self.stats.lock().await.clone()
  }

  /// 获取所有规则
  pub async fn get_rules(&self) -> Vec<ErrorHandlingRule> {
    self.rules.read().await.clone()
  }

  /// 清空规则
  pub async fn clear_rules(&self) {
    let mut rules = self.rules.write().await;
    rules.clear();
  }
}

impl Default for ErrorHandler {
  fn default() -> Self {
    Self::new()
  }
}

/// 预定义的错误处理规则
pub mod predefined_rules {
  use super::*;
  use chrono::Utc;

  /// 创建默认错误处理规则
  pub fn create_default_rules() -> Vec<ErrorHandlingRule> {
    vec![
      // 网络错误重试规则
      ErrorHandlingRule {
        rule_id: Uuid::new_v4(),
        name: "Network Error Retry".to_string(),
        error_type_filter: Some("NodeExecution".to_string()),
        error_code_filter: Some("NETWORK_ERROR".to_string()),
        node_filter: None,
        severity_filter: None,
        strategy: WorkflowErrorHandlingStrategy::Retry {
          max_retries: 3,
          retry_interval_ms: 1000,
          backoff_strategy: BackoffStrategy::Exponential { base_delay_ms: 1000, max_delay_ms: 30000, multiplier: 2.0 },
        },
        enabled: true,
        priority: 100,
        created_at: Utc::now(),
      },
      // 超时错误重试规则
      ErrorHandlingRule {
        rule_id: Uuid::new_v4(),
        name: "Timeout Error Retry".to_string(),
        error_type_filter: Some("NodeExecution".to_string()),
        error_code_filter: Some("TIMEOUT".to_string()),
        node_filter: None,
        severity_filter: Some(ErrorSeverity::Error),
        strategy: WorkflowErrorHandlingStrategy::Retry {
          max_retries: 2,
          retry_interval_ms: 5000,
          backoff_strategy: BackoffStrategy::Linear,
        },
        enabled: true,
        priority: 90,
        created_at: Utc::now(),
      },
      // 配置错误终止规则
      ErrorHandlingRule {
        rule_id: Uuid::new_v4(),
        name: "Configuration Error Terminate".to_string(),
        error_type_filter: Some("Configuration".to_string()),
        error_code_filter: None,
        node_filter: None,
        severity_filter: None,
        strategy: WorkflowErrorHandlingStrategy::TerminateWorkflow,
        enabled: true,
        priority: 200,
        created_at: Utc::now(),
      },
      // 权限错误终止规则
      ErrorHandlingRule {
        rule_id: Uuid::new_v4(),
        name: "Permission Error Terminate".to_string(),
        error_type_filter: Some("NodeExecution".to_string()),
        error_code_filter: Some("PERMISSION_DENIED".to_string()),
        node_filter: None,
        severity_filter: Some(ErrorSeverity::Error),
        strategy: WorkflowErrorHandlingStrategy::TerminateWorkflow,
        enabled: true,
        priority: 190,
        created_at: Utc::now(),
      },
      // 致命错误终止规则
      ErrorHandlingRule {
        rule_id: Uuid::new_v4(),
        name: "Fatal Error Terminate".to_string(),
        error_type_filter: None,
        error_code_filter: None,
        node_filter: None,
        severity_filter: Some(ErrorSeverity::Fatal),
        strategy: WorkflowErrorHandlingStrategy::TerminateWorkflow,
        enabled: true,
        priority: 255,
        created_at: Utc::now(),
      },
    ]
  }
}
