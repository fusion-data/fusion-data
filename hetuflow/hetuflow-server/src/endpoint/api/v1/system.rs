use axum::{extract::State, routing::get};
use fusion_web::{Router, WebResult, ok_json};

use crate::{application::ServerApplication, model::HealthStatus};

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/health", get(health))
    .route("/info", get(info))
    .route("/metrics", get(metrics))
}

async fn health(State(app): State<ServerApplication>) -> WebResult<HealthStatus> {
  let body = app.health_status().await?;
  ok_json!(body)
}

async fn info(State(app): State<ServerApplication>) -> WebResult<serde_json::Value> {
  let info = app.agent_stats().await?;
  ok_json!(info)
}

async fn metrics() -> WebResult<serde_json::Value> {
  // TODO:
  ok_json!(serde_json::json!({"ok": true}))
}
