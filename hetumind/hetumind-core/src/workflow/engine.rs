use async_trait::async_trait;

use super::{
  ExecutionContext, ExecutionDataMap, ExecutionId, ExecutionResult, ExecutionStatus, NodeName, WorkflowErrorData,
  WorkflowExecutionError, WorkflowId,
};

#[async_trait]
pub trait WorkflowEngine: Send + Sync {
  /// 执行工作流
  async fn execute_workflow(
    &self,
    trigger_data: (NodeName, ExecutionDataMap),
    context: &ExecutionContext,
  ) -> Result<ExecutionResult, WorkflowExecutionError>;

  /// 暂停执行
  async fn pause_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError>;

  /// 恢复执行
  async fn resume_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError>;

  /// 取消执行
  async fn cancel_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError>;

  /// 获取执行状态
  async fn get_execution_status(&self, execution_id: &ExecutionId) -> Result<ExecutionStatus, WorkflowExecutionError>;

  /// 执行错误工作流
  async fn execute_error_workflow(
    &self,
    error_data: WorkflowErrorData,
    error_workflow_id: Option<WorkflowId>,
  ) -> Result<ExecutionResult, WorkflowExecutionError>;
}
