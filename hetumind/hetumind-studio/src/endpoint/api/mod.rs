use axum::Router;
use fusion_core::application::Application;

pub mod auth;
pub mod v1;

pub fn routes() -> Router<Application> {
  Router::new().nest("/v1", v1::v1_routes()).nest("/auth", auth::auth_routes())
}
