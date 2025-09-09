use axum::{
  Router,
  extract::Path,
  response::Json,
  routing::{get, post},
};
use fusion_core::application::Application;
use fusion_web::{WebResult, ok_json};
use hetumind_core::workflow::{Execution, ExecutionData, ExecutionForQuery, ExecutionId};
use modelsql::page::PageResult;

use crate::domain::workflow::ExecutionSvc;

pub fn routes() -> Router<Application> {
  Router::new()
    .route("/query", post(query_executions))
    .route("/{id}", get(get_execution))
    .route("/{id}/cancel", post(cancel_execution))
    .route("/{id}/retry", post(retry_execution))
    .route("/{id}/logs", get(get_execution_logs))
}

// --- API Data Models ---

/// The standard response for a single execution.
/// It uses the core execution model directly.
pub type ExecutionResponse = Execution;

/// The response for execution logs.
pub type ExecutionLogResponse = Vec<ExecutionData>;

// --- API Handlers ---

/// 查询执行历史
pub async fn query_executions(
  execution_svc: ExecutionSvc,
  Json(input): Json<ExecutionForQuery>,
) -> WebResult<PageResult<ExecutionResponse>> {
  let res = execution_svc.query_executions(input).await?;
  ok_json!(res)
}

/// 获取执行详情
pub async fn get_execution(
  execution_svc: ExecutionSvc,
  Path(execution_id): Path<ExecutionId>,
) -> WebResult<ExecutionResponse> {
  let res = execution_svc.find_execution_by_id(execution_id).await?;
  ok_json!(res)
}

/// 取消执行
pub async fn cancel_execution(execution_svc: ExecutionSvc, Path(execution_id): Path<ExecutionId>) -> WebResult<()> {
  execution_svc.cancel_execution(execution_id).await?;
  ok_json!()
}

/// 重试执行
pub async fn retry_execution(execution_svc: ExecutionSvc, Path(execution_id): Path<ExecutionId>) -> WebResult<()> {
  execution_svc.retry_execution(execution_id).await?;
  ok_json!()
}

/// 获取执行日志
pub async fn get_execution_logs(
  execution_svc: ExecutionSvc,
  Path(execution_id): Path<ExecutionId>,
) -> WebResult<ExecutionLogResponse> {
  let res = execution_svc.logs(execution_id).await?;
  ok_json!(res)
}
