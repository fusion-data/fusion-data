use axum::{
  extract::{Path, State},
  response::Json,
};
use fusions::common::model::IdI64Result;
use fusions::core::application::Application;
use fusions::web::{WebError, WebResult, ok_json};

use fusionsql::page::PageResult;
use jieyuan_core::model::{NamespaceEntity, NamespaceForCreate, NamespaceForPage, NamespaceForUpdate};
use utoipa_axum::router::OpenApiRouter;

use crate::namespace::NamespaceSvc;

// HTTP Handlers
pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    // CRUD operations
    .routes(utoipa_axum::routes!(create_namespace))
    .routes(utoipa_axum::routes!(list_namespaces))
    .routes(utoipa_axum::routes!(get_namespace))
    .routes(utoipa_axum::routes!(update_namespace))
    // Status management operations
    .routes(utoipa_axum::routes!(enable_namespace))
    .routes(utoipa_axum::routes!(disable_namespace))
    // Additional operations
    .routes(utoipa_axum::routes!(get_namespace_by_name))
    .routes(utoipa_axum::routes!(page_namespaces))
    .routes(utoipa_axum::routes!(count_namespaces))
}

/// Create a new namespace
#[utoipa::path(
  post,
  path = "/",
  request_body = NamespaceForCreate,
  responses(
    (status = 201, description = "Namespace created successfully", body = i64),
    (status = 400, description = "Bad request"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Namespaces"
)]
pub async fn create_namespace(
  namespace_svc: NamespaceSvc,
  Json(req): Json<NamespaceForCreate>,
) -> WebResult<IdI64Result> {
  let id = namespace_svc.create(req).await?;
  ok_json!(IdI64Result::new(id))
}

/// Get namespace by ID
#[utoipa::path(
  get,
  path = "/{id}",
  params(
    ("id" = i64, Path, description = "Namespace ID")
  ),
  responses(
    (status = 200, description = "Namespace found", body = NamespaceEntity),
    (status = 404, description = "Namespace not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Namespaces"
)]
pub async fn get_namespace(namespace_svc: NamespaceSvc, Path(id): Path<i64>) -> WebResult<NamespaceEntity> {
  let namespace = namespace_svc.get(id).await?.ok_or_else(|| WebError::new_with_code(404, "Namespace not found"))?;
  ok_json!(namespace)
}

/// Get namespace by name
#[utoipa::path(
  get,
  path = "/by-name/{name}",
  params(
    ("name" = String, Path, description = "Namespace name")
  ),
  responses(
    (status = 200, description = "Namespace found", body = Option<NamespaceEntity>),
    (status = 404, description = "Namespace not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Namespaces"
)]
pub async fn get_namespace_by_name(
  namespace_svc: NamespaceSvc,
  Path(name): Path<String>,
) -> WebResult<NamespaceEntity> {
  let namespace = namespace_svc
    .get_by_name(&name)
    .await?
    .ok_or_else(|| WebError::new_with_code(404, "Namespace not found"))?;
  ok_json!(namespace)
}

/// Update namespace
#[utoipa::path(
  put,
  path = "/{id}",
  params(
    ("id" = i64, Path, description = "Namespace ID")
  ),
  request_body = NamespaceForUpdate,
  responses(
    (status = 200, description = "Namespace updated successfully"),
    (status = 400, description = "Bad request"),
    (status = 404, description = "Namespace not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Namespaces"
)]
pub async fn update_namespace(
  namespace_svc: NamespaceSvc,
  Path(id): Path<i64>,
  Json(req): Json<NamespaceForUpdate>,
) -> WebResult<()> {
  namespace_svc.update(id, req).await.map_err(|e| WebError::bad_request(e.to_string()))?;
  ok_json!()
}

/// Get paginated list of namespaces
#[utoipa::path(
  post,
  path = "/page",
  request_body = NamespaceForPage,
  responses(
    (status = 200, description = "Namespace list retrieved successfully", body = PageResult<NamespaceEntity>),
    (status = 400, description = "Bad request"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Namespaces"
)]
pub async fn page_namespaces(
  namespace_svc: NamespaceSvc,
  Json(req): Json<NamespaceForPage>,
) -> WebResult<PageResult<NamespaceEntity>> {
  let result = namespace_svc.page(req).await.map_err(|e| WebError::bad_request(e.to_string()))?;
  ok_json!(result)
}

/// Get all namespaces for current tenant
#[utoipa::path(
  get,
  path = "/",
  responses(
    (status = 200, description = "Namespace list retrieved successfully", body = Vec<NamespaceEntity>),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Namespaces"
)]
pub async fn list_namespaces(
  State(_app): State<Application>,
  namespace_svc: NamespaceSvc,
) -> WebResult<Vec<NamespaceEntity>> {
  let namespaces = namespace_svc.list_by_tenant().await.map_err(|e| WebError::bad_request(e.to_string()))?;
  ok_json!(namespaces)
}

/// Get namespace count for current tenant
#[utoipa::path(
  get,
  path = "/count",
  responses(
    (status = 200, description = "Namespace count retrieved successfully", body = u64),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Namespaces"
)]
pub async fn count_namespaces(State(_app): State<Application>, namespace_svc: NamespaceSvc) -> WebResult<u64> {
  let count = namespace_svc.count_by_tenant().await?;
  ok_json!(count)
}

/// Enable namespace
#[utoipa::path(
  post,
  path = "/{id}/enable",
  params(
    ("id" = i64, Path, description = "Namespace ID")
  ),
  responses(
    (status = 200, description = "Namespace enabled successfully"),
    (status = 404, description = "Namespace not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Namespaces"
)]
pub async fn enable_namespace(namespace_svc: NamespaceSvc, Path(id): Path<i64>) -> WebResult<()> {
  namespace_svc.enable(id).await.map_err(|e| WebError::bad_request(e.to_string()))?;
  ok_json!(())
}

/// Disable namespace
#[utoipa::path(
  post,
  path = "/{id}/disable",
  params(
    ("id" = i64, Path, description = "Namespace ID")
  ),
  responses(
    (status = 200, description = "Namespace disabled successfully"),
    (status = 404, description = "Namespace not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Namespaces"
)]
pub async fn disable_namespace(namespace_svc: NamespaceSvc, Path(id): Path<i64>) -> WebResult<()> {
  namespace_svc.disable(id).await.map_err(|e| WebError::bad_request(e.to_string()))?;
  ok_json!(())
}
