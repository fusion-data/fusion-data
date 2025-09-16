pub mod v1;

use fusion_core::application::Application;
use utoipa_axum::router::OpenApiRouter;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new().nest("/v1", v1::routes())
}