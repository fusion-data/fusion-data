pub mod auth;
pub mod v1;

use ultimates::core::application::Application;
use utoipa_axum::router::OpenApiRouter;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new().nest("/v1", v1::routes()).nest("/auth", auth::routes())
}
