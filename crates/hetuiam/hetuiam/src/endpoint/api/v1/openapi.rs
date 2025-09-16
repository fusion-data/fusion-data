use fusion_core::application::Application;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

/// HetuIAM API 文档配置
#[derive(OpenApi)]
#[openapi(info(
  title = "HetuIAM API",
  version = "1.0.0",
  description = "身份认证与访问管理系统 API 文档",
  contact(name = "HetuIAM Team", email = "support@hetuiam.com"),
))]
pub struct ApiDoc;

/// 创建带有 OpenAPI 文档的路由器
pub fn create_openapi_router() -> OpenApiRouter<Application> {
  OpenApiRouter::with_openapi(ApiDoc::openapi())
    .merge(super::auth::routes())
    .merge(super::users::routes())
    .merge(super::roles::routes())
    .merge(super::permissions::routes())
    .merge(super::policies::routes())
}

/// 获取 OpenAPI JSON 文档
pub fn get_openapi_json() -> String {
  ApiDoc::openapi().to_pretty_json().unwrap()
}
