use axum::{
  Json,
  extract::Path,
  routing::{get, post},
};
use modelsql::page::PageResult;
use fusion_core::IdUuidResult;
use fusion_web::{Router, WebResult, ok_json};
use uuid::Uuid;

use hetuflow_core::{
  models::{JobEntity, JobForCreate, JobForQuery, JobForUpdate},
  types::JobStatus,
};

use crate::{application::ServerApplication, service::JobSvc};

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/query", post(query_jobs))
    .route("/create", post(create_job))
    .route("/{id}", get(get_job).delete(delete_job))
    .route("/{id}/update", post(update_job))
    .route("/{id}/enable", post(enable_job))
    .route("/{id}/disable", post(disable_job))
}

async fn query_jobs(job_svc: JobSvc, Json(input): Json<JobForQuery>) -> WebResult<PageResult<JobEntity>> {
  let result = job_svc.query(input).await?;
  ok_json!(result)
}

async fn create_job(job_svc: JobSvc, Json(input): Json<JobForCreate>) -> WebResult<IdUuidResult> {
  let id = job_svc.create(input).await?;
  ok_json!(IdUuidResult::from(id))
}

async fn get_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<Option<JobEntity>> {
  let result = job_svc.get_by_id(&id).await?;
  ok_json!(result)
}

async fn update_job(job_svc: JobSvc, Path(id): Path<Uuid>, Json(input): Json<JobForUpdate>) -> WebResult<()> {
  job_svc.update_by_id(&id, input).await?;
  ok_json!()
}

async fn delete_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  job_svc.delete_by_id(&id).await?;
  ok_json!()
}

async fn enable_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  job_svc.update_status(&id, JobStatus::Enabled).await?;
  ok_json!()
}

async fn disable_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  job_svc.update_status(&id, JobStatus::Disabled).await?;
  ok_json!()
}
