use axum::extract::State;
use fusion_web::{WebResult, ok_json};
use utoipa_axum::router::OpenApiRouter;

use crate::{application::ServerApplication, model::HealthStatus};

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(health))
    .routes(utoipa_axum::routes!(info))
    .routes(utoipa_axum::routes!(metrics))
}

#[utoipa::path(
  get,
  path = "/health",
  responses(
    (status = 200, description = "Get system health status", body = HealthStatus)
  )
)]
async fn health(State(app): State<ServerApplication>) -> WebResult<HealthStatus> {
  let body = app.health_status().await?;
  ok_json!(body)
}

#[utoipa::path(
  get,
  path = "/info",
  responses(
    (status = 200, description = "Get system information", body = serde_json::Value)
  )
)]
async fn info(State(app): State<ServerApplication>) -> WebResult<serde_json::Value> {
  let info = app.agent_stats().await?;
  ok_json!(info)
}

#[utoipa::path(
  get,
  path = "/metrics",
  responses(
    (status = 200, description = "Get system metrics", body = serde_json::Value)
  )
)]
async fn metrics() -> WebResult<serde_json::Value> {
  // TODO:
  ok_json!(serde_json::json!({"ok": true}))
}
