use axum::{Json, extract::Path};
use fusion_common::model::IdUuidResult;
use fusion_web::{WebResult, ok_json};
use hetuflow_core::models::{SchedTask, TaskForCreate, TaskForQuery, TaskForUpdate};
use fusionsql::page::PageResult;
use serde_json::Value;
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use crate::{application::ServerApplication, service::TaskSvc};

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(query_tasks))
    .routes(utoipa_axum::routes!(create_task))
    .routes(utoipa_axum::routes!(get_task))
    .routes(utoipa_axum::routes!(update_task))
    .routes(utoipa_axum::routes!(delete_task))
    .routes(utoipa_axum::routes!(retry_task))
    .routes(utoipa_axum::routes!(cancel_task))
}

// parent: /api/v1/tasks
#[utoipa::path(
  post,
  path = "/query",
  request_body = TaskForQuery,
  responses(
    (status = 200, body = PageResult<SchedTask>)
  )
)]
async fn query_tasks(task_svc: TaskSvc, Json(input): Json<TaskForQuery>) -> WebResult<PageResult<SchedTask>> {
  let page_result = task_svc.page(input).await?;
  ok_json!(page_result)
}

#[utoipa::path(
  post,
  path = "/create",
  request_body = TaskForCreate,
  responses(
    (status = 200, description = "Success", body = IdUuidResult)
  )
)]
async fn create_task(task_svc: TaskSvc, Json(input): Json<TaskForCreate>) -> WebResult<IdUuidResult> {
  let id = task_svc.create_task(input).await?;
  ok_json!(id.into())
}

#[utoipa::path(
  get,
  path = "/{id}",
  params(
    ("id" = Uuid, Path, description = "Task ID")
  ),
  responses(
    (status = 200, description = "Success", body = Option<SchedTask>)
  )
)]
async fn get_task(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Option<SchedTask>> {
  let task = task_svc.get_by_id(id).await?;
  ok_json!(task)
}
#[utoipa::path(
  post,
  path = "/{id}/update",
  params(
    ("id" = Uuid, Path, description = "Task ID")
  ),
  request_body = TaskForUpdate,
  responses(
    (status = 200, description = "Success", body = Value)
  )
)]
async fn update_task(task_svc: TaskSvc, Path(id): Path<Uuid>, Json(input): Json<TaskForUpdate>) -> WebResult<Value> {
  task_svc.update_task(id, input).await?;
  ok_json!()
}
#[utoipa::path(
  delete,
  path = "/{id}",
  params(
    ("id" = Uuid, Path, description = "Task ID")
  ),
  responses(
    (status = 200, description = "Success", body = Value)
  )
)]
async fn delete_task(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Value> {
  task_svc.delete_task(id).await?;
  ok_json!()
}
#[utoipa::path(
  post,
  path = "/{id}/retry",
  params(
    ("id" = Uuid, Path, description = "Task ID")
  ),
  responses(
    (status = 200, description = "Success", body = Value)
  )
)]
async fn retry_task(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Value> {
  task_svc.retry_task(id).await?;
  ok_json!()
}
#[utoipa::path(
  post,
  path = "/{id}/cancel",
  params(
    ("id" = Uuid, Path, description = "Task ID")
  ),
  responses(
    (status = 200, description = "Success", body = Value)
  )
)]
async fn cancel_task(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Value> {
  task_svc.cancel_task(id).await?;
  ok_json!()
}
