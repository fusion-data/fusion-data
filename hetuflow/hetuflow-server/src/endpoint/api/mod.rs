pub mod v1;

use utoipa_axum::router::OpenApiRouter;

use crate::application::ServerApplication;

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new().nest("/v1", v1::routes())
}
