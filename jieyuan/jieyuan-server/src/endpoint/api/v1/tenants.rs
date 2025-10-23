use axum::{
  extract::{Path, State},
  response::Json,
};
use fusion_common::model::IdI64Result;
use fusion_core::application::Application;
use fusion_web::{WebError, WebResult, ok_json};
use fusionsql::page::PageResult;
use jieyuan_core::model::{Tenant, TenantForCreate, TenantForPage, TenantForUpdate};
use utoipa_axum::router::OpenApiRouter;

use crate::tenant::TenantSvc;

// HTTP Handlers
pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    // CRUD operations
    .routes(utoipa_axum::routes!(create_tenant))
    .routes(utoipa_axum::routes!(get_tenant))
    .routes(utoipa_axum::routes!(update_tenant))
    // Status management operations
    .routes(utoipa_axum::routes!(enable_tenant))
    .routes(utoipa_axum::routes!(disable_tenant))
    // Additional operations
    .routes(utoipa_axum::routes!(get_tenant_by_name))
    .routes(utoipa_axum::routes!(page_tenants))
    .routes(utoipa_axum::routes!(count_tenants))
}

/// Create a new tenant
#[utoipa::path(
  post,
  path = "/item",
  request_body = TenantForCreate,
  responses(
    (status = 201, description = "Tenant created successfully", body = i64),
    (status = 400, description = "Bad request"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Tenants"
)]
pub async fn create_tenant(tenant_svc: TenantSvc, Json(req): Json<TenantForCreate>) -> WebResult<IdI64Result> {
  let id = tenant_svc.create(req).await?;
  ok_json!(IdI64Result::new(id))
}

/// Get tenant by ID
#[utoipa::path(
  get,
  path = "/item/{id}",
  params(
    ("id" = i64, Path, description = "Tenant ID")
  ),
  responses(
    (status = 200, description = "Tenant found", body = Tenant),
    (status = 404, description = "Tenant not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Tenants"
)]
pub async fn get_tenant(tenant_svc: TenantSvc, Path(id): Path<i64>) -> WebResult<Tenant> {
  let tenant = tenant_svc.get(id).await?.ok_or_else(|| WebError::new_with_code(404, "Tenant not found"))?;
  ok_json!(tenant)
}

/// Get tenant by name
#[utoipa::path(
  get,
  path = "/by-name/{name}",
  params(
    ("name" = String, Path, description = "Tenant name")
  ),
  responses(
    (status = 200, description = "Tenant found", body = Tenant),
    (status = 404, description = "Tenant not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Tenants"
)]
pub async fn get_tenant_by_name(tenant_svc: TenantSvc, Path(name): Path<String>) -> WebResult<Tenant> {
  let tenant = tenant_svc
    .get_by_name(&name)
    .await?
    .ok_or_else(|| WebError::new_with_code(404, "Tenant not found"))?;
  ok_json!(tenant)
}

/// Update tenant
#[utoipa::path(
  put,
  path = "/item/{id}",
  params(
    ("id" = i64, Path, description = "Tenant ID")
  ),
  request_body = TenantForUpdate,
  responses(
    (status = 200, description = "Tenant updated successfully"),
    (status = 400, description = "Bad request"),
    (status = 404, description = "Tenant not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Tenants"
)]
pub async fn update_tenant(
  tenant_svc: TenantSvc,
  Path(id): Path<i64>,
  Json(req): Json<TenantForUpdate>,
) -> WebResult<()> {
  tenant_svc.update(id, req).await.map_err(|e| WebError::bad_request(e.to_string()))?;
  ok_json!(())
}

/// Get paginated list of tenants
#[utoipa::path(
  post,
  path = "/page",
  request_body = TenantForPage,
  responses(
    (status = 200, description = "Tenant list retrieved successfully", body = PageResult<Tenant>),
    (status = 400, description = "Bad request"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Tenants"
)]
pub async fn page_tenants(tenant_svc: TenantSvc, Json(req): Json<TenantForPage>) -> WebResult<PageResult<Tenant>> {
  let result = tenant_svc.page(req).await.map_err(|e| WebError::bad_request(e.to_string()))?;
  ok_json!(result)
}

/// Get active tenant count
#[utoipa::path(
  get,
  path = "/count",
  responses(
    (status = 200, description = "Tenant count retrieved successfully", body = u64),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Tenants"
)]
pub async fn count_tenants(State(_app): State<Application>, tenant_svc: TenantSvc) -> WebResult<u64> {
  let count = tenant_svc.count_active().await?;
  ok_json!(count)
}

/// Enable tenant
#[utoipa::path(
  post,
  path = "/item/{id}/enable",
  params(
    ("id" = i64, Path, description = "Tenant ID")
  ),
  responses(
    (status = 200, description = "Tenant enabled successfully"),
    (status = 404, description = "Tenant not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Tenants"
)]
pub async fn enable_tenant(tenant_svc: TenantSvc, Path(id): Path<i64>) -> WebResult<()> {
  tenant_svc.enable(id).await.map_err(|e| WebError::bad_request(e.to_string()))?;
  ok_json!(())
}

/// Disable tenant
#[utoipa::path(
  post,
  path = "/item/{id}/disable",
  params(
    ("id" = i64, Path, description = "Tenant ID")
  ),
  responses(
    (status = 200, description = "Tenant disabled successfully"),
    (status = 404, description = "Tenant not found"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden")
  ),
  tag = "Tenants"
)]
pub async fn disable_tenant(tenant_svc: TenantSvc, Path(id): Path<i64>) -> WebResult<()> {
  tenant_svc.disable(id).await.map_err(|e| WebError::bad_request(e.to_string()))?;
  ok_json!(())
}
