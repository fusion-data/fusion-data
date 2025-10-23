use axum::{Json, extract::State};
use fusions::core::application::Application;
use fusions::web::{WebResult, ok_json};
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{TenantUser, TenantUserForCreate, TenantUserForUpdate, TenantUserStatus};

use crate::user::UserSvc;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(link_user_to_tenant))
    .routes(utoipa_axum::routes!(unlink_user_from_tenant))
    .routes(utoipa_axum::routes!(update_user_tenant_status))
    .routes(utoipa_axum::routes!(get_user_active_tenants))
}

/// 关联用户到租户
#[utoipa::path(
  post,
  path = "/tenant-users/link",
  request_body = TenantUserForCreate,
  responses(
    (status = 200, description = "关联成功"),
    (status = 400, description = "请求参数错误"),
    (status = 404, description = "用户或租户不存在")
  ),
  tag = "租户用户管理"
)]
async fn link_user_to_tenant(
  State(app): State<Application>,
  Json(req): Json<TenantUserForCreate>,
) -> WebResult<serde_json::Value> {
  let mm = app.get_component::<fusionsql::ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm);

  let status = req.status.unwrap_or(TenantUserStatus::Active);
  user_svc.link_user_to_tenant(req.user_id, req.tenant_id, status).await?;

  ok_json!(serde_json::Value::Object(serde_json::Map::new()))
}

/// 取消用户与租户的关联
#[utoipa::path(
  delete,
  path = "/tenant-users/{user_id}/{tenant_id}",
  params(
    ("user_id" = i64, Path, description = "用户ID"),
    ("tenant_id" = i64, Path, description = "租户ID")
  ),
  responses(
    (status = 200, description = "取消关联成功"),
    (status = 404, description = "关联不存在")
  ),
  tag = "租户用户管理"
)]
async fn unlink_user_from_tenant(
  State(app): State<Application>,
  axum::extract::Path((user_id, tenant_id)): axum::extract::Path<(i64, i64)>,
) -> WebResult<serde_json::Value> {
  let mm = app.get_component::<fusionsql::ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm);

  user_svc.unlink_user_from_tenant(user_id, tenant_id).await?;

  ok_json!(serde_json::Value::Object(serde_json::Map::new()))
}

/// 更新用户在租户中的状态
#[utoipa::path(
  put,
  path = "/tenant-users/{user_id}/{tenant_id}/status",
  params(
    ("user_id" = i64, Path, description = "用户ID"),
    ("tenant_id" = i64, Path, description = "租户ID")
  ),
  request_body = TenantUserForUpdate,
  responses(
    (status = 200, description = "更新成功"),
    (status = 404, description = "关联不存在")
  ),
  tag = "租户用户管理"
)]
async fn update_user_tenant_status(
  State(app): State<Application>,
  axum::extract::Path((user_id, tenant_id)): axum::extract::Path<(i64, i64)>,
  Json(req): Json<TenantUserForUpdate>,
) -> WebResult<serde_json::Value> {
  let mm = app.get_component::<fusionsql::ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm);

  let status = req.status.ok_or_else(|| fusions::core::DataError::bad_request("status is required"))?;

  user_svc.update_user_tenant_status(user_id, tenant_id, status).await?;

  ok_json!(serde_json::Value::Object(serde_json::Map::new()))
}

/// 获取用户的所有活跃租户关联
#[utoipa::path(
  get,
  path = "/tenant-users/user/{user_id}/active",
  params(
    ("user_id" = i64, Path, description = "用户ID")
  ),
  responses(
    (status = 200, description = "查询成功", body = Vec<TenantUser>),
    (status = 404, description = "用户不存在")
  ),
  tag = "租户用户管理"
)]
async fn get_user_active_tenants(
  State(app): State<Application>,
  axum::extract::Path(user_id): axum::extract::Path<i64>,
) -> WebResult<Vec<TenantUser>> {
  let mm = app.get_component::<fusionsql::ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm);

  let tenants = user_svc.get_user_active_tenants(user_id).await?;
  ok_json!(tenants)
}
