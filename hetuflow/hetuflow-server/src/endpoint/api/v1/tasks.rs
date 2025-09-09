use axum::{
  Json,
  extract::Path,
  routing::{get, post},
};
use fusion_core::IdUuidResult;
use fusion_web::{Router, WebResult, ok_json};
use hetuflow_core::models::{SchedTask, TaskForCreate, TaskForQuery, TaskForUpdate};
use modelsql::page::PageResult;
use serde_json::Value;
use uuid::Uuid;

use crate::{application::ServerApplication, service::TaskSvc};

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/query", post(query_tasks))
    .route("/create", post(create_task))
    .route("/{id}", get(get_task).delete(delete_task))
    .route("/{id}/update", post(update_task))
    .route("/{id}/retry", post(retry_task))
    .route("/{id}/cancel", post(cancel_task))
}

async fn query_tasks(task_svc: TaskSvc, Json(input): Json<TaskForQuery>) -> WebResult<PageResult<SchedTask>> {
  let page_result = task_svc.page(input).await?;
  ok_json!(page_result)
}
async fn create_task(task_svc: TaskSvc, Json(input): Json<TaskForCreate>) -> WebResult<IdUuidResult> {
  let id = task_svc.create_task(input).await?;
  ok_json!(id.into())
}
async fn get_task(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Option<SchedTask>> {
  let task = task_svc.get_by_id(id).await?;
  ok_json!(task)
}
async fn update_task(task_svc: TaskSvc, Path(id): Path<Uuid>, Json(input): Json<TaskForUpdate>) -> WebResult<Value> {
  task_svc.update_task(id, input).await?;
  ok_json!()
}
async fn delete_task(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Value> {
  task_svc.delete_task(id).await?;
  ok_json!()
}
async fn retry_task(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Value> {
  task_svc.retry_task(id).await?;
  ok_json!()
}
async fn cancel_task(task_svc: TaskSvc, Path(id): Path<Uuid>) -> WebResult<Value> {
  task_svc.cancel_task(id).await?;
  ok_json!()
}
