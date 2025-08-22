// 复用 hetumind-core 中已有的数据模型
pub use hetumind_core::workflow::{
  ExecuteWorkflowRequest,
  // 执行相关
  Execution,
  ExecutionData,
  ExecutionId,
  ExecutionIdResponse,
  ExecutionStatus,
  ValidateWorkflowRequest,
  ValidateWorkflowResponse,
  // 工作流相关
  Workflow,
  WorkflowId,
  WorkflowStatus,
};

pub use modelsql::page::PageResult;

use serde::{Deserialize, Serialize};

/// API 标准响应格式
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
  pub success: bool,
  pub data: Option<T>,
  pub error: Option<String>,
}

impl<T> ApiResponse<T> {
  pub fn success(data: T) -> Self {
    Self { success: true, data: Some(data), error: None }
  }

  pub fn error(message: impl Into<String>) -> Self {
    Self { success: false, data: None, error: Some(message.into()) }
  }
}

/// API 错误响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
  pub message: String,
  pub code: Option<String>,
  pub details: Option<serde_json::Value>,
}

/// 工作流列表查询响应
pub type WorkflowListResponse = PageResult<Workflow>;

/// 执行列表查询响应
pub type ExecutionListResponse = PageResult<Execution>;

/// 执行日志响应
pub type ExecutionLogsResponse = Vec<ExecutionData>;
