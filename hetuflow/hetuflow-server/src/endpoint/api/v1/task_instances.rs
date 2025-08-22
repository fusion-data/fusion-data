use axum::{
  Json,
  extract::Path,
  routing::{get, post},
};
use hetuflow_core::models::{
  TaskInstanceEntity, TaskInstanceForCreate, TaskInstanceForQuery, TaskInstanceForUpdate,
};
use modelsql::page::PageResult;
use serde_json::Value;
use ultimate_web::{Router, WebResult, ok_json};
use uuid::Uuid;

use crate::{application::ServerApplication, service::TaskSvc};

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/query", post(query_task_instances))
    .route("/create", post(create_task_instance))
    .route("/{id}", get(get_task_instance).delete(delete_task_instance))
    .route("/{id}/update", post(update_task_instance))
}

async fn query_task_instances(
  task_svc: TaskSvc,
  Json(input): Json<TaskInstanceForQuery>,
) -> WebResult<PageResult<TaskInstanceEntity>> {
  let page_result = task_svc.find_task_instances_page(input).await?;
  ok_json!(page_result)
}

async fn create_task_instance(task_svc: TaskSvc, Json(input): Json<TaskInstanceForCreate>) -> WebResult<Uuid> {
  let id = task_svc.create_task_instance(input).await?;
  ok_json!(id)
}

async fn get_task_instance(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Option<TaskInstanceEntity>> {
  let task_instance = task_svc.find_task_instance(id).await?;
  ok_json!(task_instance)
}

async fn delete_task_instance(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Value> {
  task_svc.delete_task_instance(id).await?;
  ok_json!()
}

async fn update_task_instance(
  task_svc: TaskSvc,
  Path(id): Path<Uuid>,
  Json(input): Json<TaskInstanceForUpdate>,
) -> WebResult<Value> {
  task_svc.update_task_instance(id, input).await?;
  ok_json!()
}
