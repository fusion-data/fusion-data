use axum::{Json, extract::Path};
use fusion_web::{WebResult, ok_json};
use fusionsql::page::PageResult;
use serde_json::Value;

use hetuflow_core::models::{SchedServer, ServerForQuery, ServerForUpdate};
use utoipa_axum::router::OpenApiRouter;

use crate::{application::ServerApplication, service::ServerSvc};

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(query_servers))
    .routes(utoipa_axum::routes!(get_server))
    .routes(utoipa_axum::routes!(update_server))
    .routes(utoipa_axum::routes!(delete_server))
}

#[utoipa::path(
  post,
  path = "/query",
  request_body = ServerForQuery,
  responses(
    (status = 200, body = PageResult<SchedServer>)
  ),
  tag = "Servers"
)]
async fn query_servers(server_svc: ServerSvc, Json(input): Json<ServerForQuery>) -> WebResult<PageResult<SchedServer>> {
  let result = server_svc.page(input).await?;
  ok_json!(result)
}

#[utoipa::path(
  get,
  path = "/{id}",
  params(
    ("id" = String, Path, description = "Server ID")
  ),
  responses(
    (status = 200, description = "Success", body = Option<SchedServer>)
  ),
  tag = "Servers"
)]
async fn get_server(server_svc: ServerSvc, Path(id): Path<String>) -> WebResult<Option<SchedServer>> {
  let result = server_svc.get_by_id(&id).await?;
  ok_json!(result)
}

#[utoipa::path(
  post,
  path = "/{id}/update",
  params(
    ("id" = String, Path, description = "Server ID")
  ),
  request_body = ServerForUpdate,
  responses(
    (status = 200, description = "Success", body = Value)
  ),
  tag = "Servers"
)]
async fn update_server(
  server_svc: ServerSvc,
  Path(id): Path<String>,
  Json(input): Json<ServerForUpdate>,
) -> WebResult<Value> {
  server_svc.update_by_id(&id, input).await?;
  ok_json!()
}

#[utoipa::path(
  delete,
  path = "/{id}",
  params(
    ("id" = String, Path, description = "Server ID")
  ),
  responses(
    (status = 200, description = "Success", body = Value)
  ),
  tag = "Servers"
)]
async fn delete_server(server_svc: ServerSvc, Path(id): Path<String>) -> WebResult<Value> {
  server_svc.delete_by_id(&id).await?;
  ok_json!()
}
