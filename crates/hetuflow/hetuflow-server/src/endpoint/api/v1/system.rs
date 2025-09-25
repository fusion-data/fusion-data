use axum::extract::State;
use fusion_web::{WebResult, ok_json};
use utoipa_axum::router::OpenApiRouter;

use crate::{application::ServerApplication, model::SystemStatus};

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new().routes(utoipa_axum::routes!(health)).routes(utoipa_axum::routes!(metrics))
}

#[utoipa::path(
  get,
  path = "/health",
  responses(
    (status = 200, description = "Get system health status", body = SystemStatus)
  )
)]
async fn health(State(app): State<ServerApplication>) -> WebResult<SystemStatus> {
  let body = app.health_status().await?;
  ok_json!(body)
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
