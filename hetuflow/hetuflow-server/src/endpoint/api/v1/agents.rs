use axum::{Json, extract::Path};
use fusion_core::IdStringResult;
use fusion_web::{WebResult, ok_json};
use modelsql::page::PageResult;
use serde_json::Value;

use hetuflow_core::models::{AgentForCreate, AgentForQuery, AgentForUpdate, SchedAgent};
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use crate::{application::ServerApplication, service::AgentSvc};

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(query_agents))
    .routes(utoipa_axum::routes!(create_agent))
    .routes(utoipa_axum::routes!(get_agent))
    .routes(utoipa_axum::routes!(update_agent))
    .routes(utoipa_axum::routes!(delete_agent))
}

#[utoipa::path(
  post,
  path = "/query",
  request_body = AgentForQuery,
  responses(
    (status = 200, body = PageResult<SchedAgent>)
  ),
  tag = "Agents"
)]
async fn query_agents(agent_svc: AgentSvc, Json(input): Json<AgentForQuery>) -> WebResult<PageResult<SchedAgent>> {
  let result = agent_svc.query(input).await?;
  ok_json!(result)
}

#[utoipa::path(
  post,
  path = "/create",
  request_body = AgentForCreate,
  responses(
    (status = 200, description = "Success", body = IdStringResult)
  ),
  tag = "Agents"
)]
async fn create_agent(agent_svc: AgentSvc, Json(input): Json<AgentForCreate>) -> WebResult<IdStringResult> {
  let id = agent_svc.create(input).await?;
  ok_json!(IdStringResult::new(id))
}

#[utoipa::path(
  get,
  path = "/{id}",
  params(
    ("id" = Uuid, Path, description = "Agent ID")
  ),
  responses(
    (status = 200, description = "Success", body = Option<SchedAgent>)
  ),
  tag = "Agents"
)]
async fn get_agent(agent_svc: AgentSvc, Path(id): Path<Uuid>) -> WebResult<Option<SchedAgent>> {
  let result = agent_svc.get_by_id(&id).await?;
  ok_json!(result)
}

#[utoipa::path(
  post,
  path = "/{id}/update",
  params(
    ("id" = Uuid, Path, description = "Agent ID")
  ),
  request_body = AgentForUpdate,
  responses(
    (status = 200, description = "Success", body = Value)
  ),
  tag = "Agents"
)]
async fn update_agent(
  agent_svc: AgentSvc,
  Path(id): Path<Uuid>,
  Json(input): Json<AgentForUpdate>,
) -> WebResult<Value> {
  agent_svc.update_by_id(&id, input).await?;
  ok_json!()
}

#[utoipa::path(
  delete,
  path = "/{id}",
  params(
    ("id" = Uuid, Path, description = "Agent ID")
  ),
  responses(
    (status = 200, description = "Success", body = Value)
  ),
  tag = "Agents"
)]
async fn delete_agent(agent_svc: AgentSvc, Path(id): Path<Uuid>) -> WebResult<Value> {
  agent_svc.delete_by_id(&id).await?;
  ok_json!()
}
