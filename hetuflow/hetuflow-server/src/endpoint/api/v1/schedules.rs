use axum::{Json, extract::Path};
use fusion_core::IdUuidResult;
use fusion_web::{WebResult, ok_json};
use modelsql::page::PageResult;
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use hetuflow_core::models::{SchedSchedule, ScheduleForCreate, ScheduleForQuery, ScheduleForUpdate};

use crate::{application::ServerApplication, service::ScheduleSvc};

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(query_schedules))
    .routes(utoipa_axum::routes!(create_schedule))
    .routes(utoipa_axum::routes!(get_schedule))
    .routes(utoipa_axum::routes!(update_schedule))
    .routes(utoipa_axum::routes!(delete_schedule))
    .routes(utoipa_axum::routes!(get_schedulable_schedules))
}

#[utoipa::path(
  post,
  path = "/page",
  request_body = ScheduleForQuery,
  responses(
    (status = 200, description = "Query schedules successfully", body = PageResult<SchedSchedule>)
  ),
  tag = "Schedules"
)]
async fn query_schedules(
  schedule_svc: ScheduleSvc,
  Json(input): Json<ScheduleForQuery>,
) -> WebResult<PageResult<SchedSchedule>> {
  let result = schedule_svc.query(input).await?;
  ok_json!(result)
}

#[utoipa::path(
  post,
  path = "/item",
  request_body = ScheduleForCreate,
  responses(
    (status = 200, description = "Create schedule successfully", body = IdUuidResult)
  ),
  tag = "Schedules"
)]
async fn create_schedule(schedule_svc: ScheduleSvc, Json(input): Json<ScheduleForCreate>) -> WebResult<IdUuidResult> {
  let id = schedule_svc.create(input).await?;
  ok_json!(IdUuidResult::from(id))
}

#[utoipa::path(
  get,
  path = "/item/{id}",
  params(
    ("id" = Uuid, Path, description = "Schedule ID")
  ),
  responses(
    (status = 200, description = "Get schedule successfully", body = Option<SchedSchedule>)
  ),
  tag = "Schedules"
)]
async fn get_schedule(schedule_svc: ScheduleSvc, Path(id): Path<Uuid>) -> WebResult<Option<SchedSchedule>> {
  let result = schedule_svc.get_by_id(&id).await?;
  ok_json!(result)
}

#[utoipa::path(
  put,
  path = "/item/{id}",
  params(
    ("id" = Uuid, Path, description = "Schedule ID")
  ),
  request_body = ScheduleForUpdate,
  responses(
    (status = 200, description = "Update schedule successfully")
  ),
  tag = "Schedules"
)]
async fn update_schedule(
  schedule_svc: ScheduleSvc,
  Path(id): Path<Uuid>,
  Json(input): Json<ScheduleForUpdate>,
) -> WebResult<()> {
  schedule_svc.update_by_id(&id, input).await?;
  ok_json!()
}

#[utoipa::path(
  delete,
  path = "/item/{id}",
  params(
    ("id" = Uuid, Path, description = "Schedule ID")
  ),
  responses(
    (status = 200, description = "Delete schedule successfully")
  ),
  tag = "Schedules"
)]
async fn delete_schedule(schedule_svc: ScheduleSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  schedule_svc.delete_by_id(&id).await?;
  ok_json!()
}
#[utoipa::path(
  get,
  path = "/schedulable",
  responses(
    (status = 200, description = "Get schedulable schedules successfully", body = Vec<SchedSchedule>)
  ),
  tag = "Schedules"
)]
async fn get_schedulable_schedules(schedule_svc: ScheduleSvc) -> WebResult<Vec<SchedSchedule>> {
  let result = schedule_svc.find_schedulable().await?;
  ok_json!(result)
}
