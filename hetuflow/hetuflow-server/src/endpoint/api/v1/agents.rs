use axum::{
  Json,
  extract::Path,
  routing::{get, post},
};
use fusion_core::IdUuidResult;
use fusion_web::{Router, WebResult, ok_json};
use modelsql::page::PageResult;

use hetuflow_core::models::{AgentForCreate, AgentForQuery, AgentForUpdate, SchedAgent};
use uuid::Uuid;

use crate::{application::ServerApplication, service::AgentSvc};

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/query", post(query_agents))
    .route("/create", post(create_agent))
    .route("/{id}", get(get_agent).delete(delete_agent))
    .route("/{id}/update", post(update_agent))
}

async fn query_agents(agent_svc: AgentSvc, Json(input): Json<AgentForQuery>) -> WebResult<PageResult<SchedAgent>> {
  let result = agent_svc.query(input).await?;
  ok_json!(result)
}

async fn create_agent(agent_svc: AgentSvc, Json(input): Json<AgentForCreate>) -> WebResult<IdUuidResult> {
  let id = agent_svc.create(input).await?;
  ok_json!(id.into())
}

async fn get_agent(agent_svc: AgentSvc, Path(id): Path<Uuid>) -> WebResult<Option<SchedAgent>> {
  let result = agent_svc.get_by_id(&id).await?;
  ok_json!(result)
}

async fn update_agent(agent_svc: AgentSvc, Path(id): Path<Uuid>, Json(input): Json<AgentForUpdate>) -> WebResult<()> {
  agent_svc.update_by_id(&id, input).await?;
  ok_json!()
}

async fn delete_agent(agent_svc: AgentSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  agent_svc.delete_by_id(&id).await?;
  ok_json!()
}
