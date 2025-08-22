use axum::{extract::FromRequestParts, http::request::Parts};
use ultimate_web::WebError;

use crate::application::ServerApplication;
use crate::service::{AgentSvc, JobSvc, TaskSvc};

impl FromRequestParts<ServerApplication> for TaskSvc {
  type Rejection = WebError;

  async fn from_request_parts(_req: &mut Parts, state: &ServerApplication) -> Result<Self, Self::Rejection> {
    Ok(Self::new(state.mm.clone()))
  }
}

impl FromRequestParts<ServerApplication> for AgentSvc {
  type Rejection = WebError;

  async fn from_request_parts(_parts: &mut Parts, state: &ServerApplication) -> Result<Self, Self::Rejection> {
    Ok(Self::new(state.mm.clone()))
  }
}

impl FromRequestParts<ServerApplication> for JobSvc {
  type Rejection = WebError;

  async fn from_request_parts(_req: &mut Parts, state: &ServerApplication) -> Result<Self, Self::Rejection> {
    Ok(Self { mm: state.mm.clone() })
  }
}
