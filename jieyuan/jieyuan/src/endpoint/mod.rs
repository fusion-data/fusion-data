pub mod api;
pub mod oauth;

use fusion_core::application::Application;
use fusion_web::{Router, WebError};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

/// 将所有 paths + schemas 汇总到 OpenApi 上。
#[derive(OpenApi)]
#[openapi(
  info(
    title = "JieYuan API",
    version = env!("CARGO_PKG_VERSION"),
    description = "Identity and Access Management API"
  ),
  components(
    responses(
      WebError
    )
  ),
  security(
    ("bearer_auth" = [])
  )
)]
struct ApiDoc;

pub fn routes() -> Router<Application> {
  let openapi = ApiDoc::openapi();
  let (router, api) = OpenApiRouter::with_openapi(openapi)
    .nest("/api", api::routes())
    .nest("/oauth", oauth::routes())
    .split_for_parts();

  router.merge(SwaggerUi::new("/docs/swagger-ui").url("/docs/openapi.json", api))
}
