use axum::{
  Router,
  extract::Path,
  response::Json,
  routing::{get, post},
};
use fusion_common::model::IdUuidResult;
use fusion_core::application::Application;
use fusion_web::{WebResult, ok_json};
use hetumind_core::workflow::{
  ExecuteWorkflowRequest, ExecutionIdResponse, ValidateWorkflowRequest, ValidateWorkflowResponse, Workflow,
  WorkflowForCreate, WorkflowForQuery, WorkflowForUpdate, WorkflowId, WorkflowStatus,
};
use modelsql::page::PageResult;

use crate::domain::workflow::WorkflowSvc;

pub fn routes() -> Router<Application> {
  Router::new()
    .route("/", post(create_workflow))
    .route("/query", post(query_workflows))
    .route("/validate", post(validate_workflow))
    .route("/{id}", get(get_workflow).put(update_workflow).delete(delete_workflow))
    .route("/{id}/execute", post(execute_workflow))
    .route("/{id}/activate", post(activate_workflow))
    .route("/{id}/deactivate", post(deactivate_workflow))
    .route("/{id}/duplicate", post(duplicate_workflow))
}

/// 列出工作流
pub async fn query_workflows(
  workflow_svc: WorkflowSvc,
  Json(input): Json<WorkflowForQuery>,
) -> WebResult<PageResult<Workflow>> {
  let res = workflow_svc.query_workflows(input).await?;
  ok_json!(res)
}

/// 验证工作流定义
pub async fn validate_workflow(
  workflow_svc: WorkflowSvc,
  Json(input): Json<ValidateWorkflowRequest>,
) -> WebResult<ValidateWorkflowResponse> {
  let res = workflow_svc.validate_workflow_from_request(input).await?;
  ok_json!(res)
}

/// 创建或导入工作流
pub async fn create_workflow(
  workflow_svc: WorkflowSvc,
  Json(input): Json<WorkflowForCreate>,
) -> WebResult<IdUuidResult> {
  let id = workflow_svc.create(input).await?;
  ok_json!(IdUuidResult::new(id.into()))
}

/// 获取工作流详情
pub async fn get_workflow(workflow_svc: WorkflowSvc, Path(workflow_id): Path<WorkflowId>) -> WebResult<Workflow> {
  let res = workflow_svc.get_workflow(&workflow_id).await?;
  ok_json!(res)
}

/// 更新工作流
pub async fn update_workflow(
  workflow_svc: WorkflowSvc,
  Path(workflow_id): Path<WorkflowId>,
  Json(input): Json<WorkflowForUpdate>,
) -> WebResult<IdUuidResult> {
  let id = workflow_svc.update(&workflow_id, input).await?;
  ok_json!(IdUuidResult::new(id.into()))
}

/// 删除工作流
pub async fn delete_workflow(workflow_svc: WorkflowSvc, Path(workflow_id): Path<WorkflowId>) -> WebResult<()> {
  workflow_svc.delete_workflow(&workflow_id).await?;
  ok_json!()
}

/// 执行工作流
pub async fn execute_workflow(
  workflow_svc: WorkflowSvc,
  Path(workflow_id): Path<WorkflowId>,
  Json(input): Json<ExecuteWorkflowRequest>,
) -> WebResult<ExecutionIdResponse> {
  let res = workflow_svc.execute_workflow(&workflow_id, input).await?;
  ok_json!(res)
}

/// 激活工作流
pub async fn activate_workflow(workflow_svc: WorkflowSvc, Path(workflow_id): Path<WorkflowId>) -> WebResult<()> {
  let input = WorkflowForUpdate { status: Some(WorkflowStatus::Active), ..Default::default() };
  workflow_svc.update(&workflow_id, input).await?;
  ok_json!()
}

/// 停用工作流
pub async fn deactivate_workflow(workflow_svc: WorkflowSvc, Path(workflow_id): Path<WorkflowId>) -> WebResult<()> {
  let input = WorkflowForUpdate { status: Some(WorkflowStatus::Disabled), ..Default::default() };
  workflow_svc.update(&workflow_id, input).await?;
  ok_json!()
}

/// 复制工作流，成功返回新工作流的ID。
pub async fn duplicate_workflow(
  workflow_svc: WorkflowSvc,
  Path(workflow_id): Path<WorkflowId>,
) -> WebResult<IdUuidResult> {
  let new_id = workflow_svc.duplicate_workflow(&workflow_id).await?;
  ok_json!(IdUuidResult::new(new_id.into()))
}
