mod _helper;
pub mod api;

use fusion_web::Router;
use hetuflow_core::models::{SchedTask, SchedTaskInstance};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::application::ServerApplication;

/// 将所有 paths + schemas 汇总到 OpenApi 上。
#[derive(OpenApi)]
#[openapi(
    // paths(get_user, create_user),
    components(schemas(SchedTask, SchedTaskInstance)),
    // 可选：添加 info、servers 等
    info(
        title = "Hetuflow Server API",
        version = env!("CARGO_PKG_VERSION"),
        description = "An example API using axum + utoipa"
    )
)]
struct ApiDoc;

pub fn routes() -> Router<ServerApplication> {
  let openapi = ApiDoc::openapi();
  let (router, api) = OpenApiRouter::with_openapi(openapi)
  // .routes(routes!(api::routes()))
.nest("/api", api::routes())
  .split_for_parts();

  router.merge(SwaggerUi::new("/docs/swagger-ui").url("/docs/openapi.json", api))
}
