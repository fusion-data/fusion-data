use axum::{
  Json,
  extract::Path,
  routing::{get, post},
};
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
    .route("/query", post(query_jobs))
    .route("/create", post(create_job))
    .route("/{id}", get(get_job).delete(delete_job))
    .route("/{id}/update", post(update_job))
    .route("/{id}/enable", post(enable_job))
    .route("/{id}/disable", post(disable_job))
}

#[utoipa::path(
  post,
  path = "/api/v1/jobs/query",
  request_body = JobForQuery,
  responses(
    (status = 200, description = "Query jobs successfully", body = PageResult<SchedJob>)
  )
)]
async fn query_jobs(job_svc: JobSvc, Json(input): Json<JobForQuery>) -> WebResult<PageResult<SchedJob>> {
  let result = job_svc.query(input).await?;
  ok_json!(result)
}

#[utoipa::path(
  post,
  path = "/api/v1/jobs/create",
  request_body = JobForCreate,
  responses(
    (status = 200, description = "Create job successfully", body = IdUuidResult)
  )
)]
async fn create_job(job_svc: JobSvc, Json(input): Json<JobForCreate>) -> WebResult<IdUuidResult> {
  let id = job_svc.create(input).await?;
  ok_json!(IdUuidResult::from(id))
}

#[utoipa::path(
  get,
  path = "/api/v1/jobs/{id}",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  responses(
    (status = 200, description = "Get job successfully", body = Option<SchedJob>)
  )
)]
async fn get_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<Option<SchedJob>> {
  let result = job_svc.get_by_id(&id).await?;
  ok_json!(result)
}

#[utoipa::path(
  post,
  path = "/api/v1/jobs/{id}/update",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  request_body = JobForUpdate,
  responses(
    (status = 200, description = "Update job successfully")
  )
)]
async fn update_job(job_svc: JobSvc, Path(id): Path<Uuid>, Json(input): Json<JobForUpdate>) -> WebResult<()> {
  job_svc.update_by_id(&id, input).await?;
  ok_json!()
}

#[utoipa::path(
  delete,
  path = "/api/v1/jobs/{id}",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  responses(
    (status = 200, description = "Delete job successfully")
  )
)]
async fn delete_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  job_svc.delete_by_id(&id).await?;
  ok_json!()
}

#[utoipa::path(
  post,
  path = "/api/v1/jobs/{id}/enable",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  responses(
    (status = 200, description = "Enable job successfully")
  )
)]
async fn enable_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  job_svc.update_status(&id, JobStatus::Enabled).await?;
  ok_json!()
}

#[utoipa::path(
  post,
  path = "/api/v1/jobs/{id}/disable",
  params(
    ("id" = Uuid, Path, description = "Job ID")
  ),
  responses(
    (status = 200, description = "Disable job successfully")
  )
)]
async fn disable_job(job_svc: JobSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  job_svc.update_status(&id, JobStatus::Disabled).await?;
  ok_json!()
}
