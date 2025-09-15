use axum::{
  Json,
  extract::Path,
  routing::{get, post},
};
use fusion_web::{WebResult, ok_json};
use hetuflow_core::models::{SchedTaskInstance, TaskInstanceForCreate, TaskInstanceForQuery, TaskInstanceForUpdate};
use modelsql::page::PageResult;
use serde_json::Value;
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use crate::{application::ServerApplication, service::TaskSvc};

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new()
    .route("/query", post(query_task_instances))
    .route("/create", post(create_task_instance))
    .route("/{id}", get(get_task_instance).delete(delete_task_instance))
    .route("/{id}/update", post(update_task_instance))
}

#[utoipa::path(
  post,
  path = "/api/v1/task_instances/query",
  request_body = TaskInstanceForQuery,
  responses(
    (status = 200, description = "Query task instances successfully", body = PageResult<SchedTaskInstance>)
  )
)]
async fn query_task_instances(
  task_svc: TaskSvc,
  Json(input): Json<TaskInstanceForQuery>,
) -> WebResult<PageResult<SchedTaskInstance>> {
  let page_result = task_svc.find_task_instances_page(input).await?;
  ok_json!(page_result)
}

#[utoipa::path(
  post,
  path = "/api/v1/task_instances/create",
  request_body = TaskInstanceForCreate,
  responses(
    (status = 200, description = "Create task instance successfully", body = Uuid)
  )
)]
async fn create_task_instance(task_svc: TaskSvc, Json(input): Json<TaskInstanceForCreate>) -> WebResult<Uuid> {
  let id = task_svc.create_task_instance(input).await?;
  ok_json!(id)
}

#[utoipa::path(
  get,
  path = "/api/v1/task_instances/{id}",
  params(
    ("id" = Uuid, Path, description = "Task instance ID")
  ),
  responses(
    (status = 200, description = "Get task instance successfully", body = Option<SchedTaskInstance>)
  )
)]
async fn get_task_instance(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Option<SchedTaskInstance>> {
  let task_instance = task_svc.find_task_instance(id).await?;
  ok_json!(task_instance)
}

#[utoipa::path(
  delete,
  path = "/api/v1/task_instances/{id}",
  params(
    ("id" = Uuid, Path, description = "Task instance ID")
  ),
  responses(
    (status = 200, description = "Delete task instance successfully")
  )
)]
async fn delete_task_instance(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Value> {
  task_svc.delete_task_instance(id).await?;
  ok_json!()
}

#[utoipa::path(
  post,
  path = "/api/v1/task_instances/{id}/update",
  params(
    ("id" = Uuid, Path, description = "Task instance ID")
  ),
  request_body = TaskInstanceForUpdate,
  responses(
    (status = 200, description = "Update task instance successfully")
  )
)]
async fn update_task_instance(
  task_svc: TaskSvc,
  Path(id): Path<Uuid>,
  Json(input): Json<TaskInstanceForUpdate>,
) -> WebResult<Value> {
  task_svc.update_task_instance(id, input).await?;
  ok_json!()
}
