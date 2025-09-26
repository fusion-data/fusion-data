use axum::{Json, extract::Path};
use fusion_core::IdUuidResult;
use fusion_web::{WebResult, ok_json};
use modelsql::page::PageResult;
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use hetuflow_core::{
  models::{JobForCreate, JobForQuery, JobForUpdate, SchedJob},
  types::JobStatus,
};

use crate::{application::ServerApplication, service::JobSvc};

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(query_jobs))
    .routes(utoipa_axum::routes!(create_job))
    .routes(utoipa_axum::routes!(get_job))
    .routes(utoipa_axum::routes!(update_job))
    .routes(utoipa_axum::routes!(enable_job))
    .routes(utoipa_axum::routes!(disable_job))
    .routes(utoipa_axum::routes!(delete_job))
}

#[utoipa::path(
  post,
  path = "/page",
  request_body = JobForQuery,
  responses(
    (status = 200, description = "Query jobs successfully", body = PageResult<SchedJob>)
  ),
  tag = "Jobs"
)]
async fn query_jobs(job_svc: JobSvc, Json(input): Json<JobForQuery>) -> WebResult<PageResult<SchedJob>> {
  let result = job_svc.query(input).await?;
  ok_json!(result)
}

#[utoipa::path(
  post,
  path = "/item",
  request_body = JobForCreate,
  responses(
    (status = 200, description = "Create job successfully", body = IdUuidResult)
  ),
  tag = "Jobs"
)]
async fn create_job(job_svc: JobSvc, Json(input): Json<JobForCreate>) -> WebResult<IdUuidResult> {
  let id = job_svc.create(input).await?;
  ok_json!(IdUuidResult::from(id))
}

#[utoipa::path(
  get,
  path = "/item/{id}",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  responses(
    (status = 200, description = "Get job successfully", body = Option<SchedJob>)
  ),
  tag = "Jobs"
)]
async fn get_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<Option<SchedJob>> {
  let result = job_svc.get_by_id(&id).await?;
  ok_json!(result)
}

#[utoipa::path(
  put,
  path = "/item/{id}",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  request_body = JobForUpdate,
  responses(
    (status = 200, description = "Update job successfully")
  ),
  tag = "Jobs"
)]
async fn update_job(job_svc: JobSvc, Path(id): Path<Uuid>, Json(input): Json<JobForUpdate>) -> WebResult<()> {
  job_svc.update_by_id(&id, input).await?;
  ok_json!()
}

#[utoipa::path(
  delete,
  path = "/item/{id}",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  responses(
    (status = 200, description = "Delete job successfully")
  ),
  tag = "Jobs"
)]
async fn delete_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  job_svc.delete_by_id(&id).await?;
  ok_json!()
}

#[utoipa::path(
  post,
  path = "/item/{id}/enable",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  responses(
    (status = 200, description = "Enable job successfully")
  ),
  tag = "Jobs"
)]
async fn enable_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  job_svc.update_status(&id, JobStatus::Enabled).await?;
  ok_json!()
}

#[utoipa::path(
  post,
  path = "/item/{id}/disable",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  responses(
    (status = 200, description = "Disable job successfully")
  ),
  tag = "Jobs"
)]
async fn disable_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  job_svc.update_status(&id, JobStatus::Disabled).await?;
  ok_json!()
}
