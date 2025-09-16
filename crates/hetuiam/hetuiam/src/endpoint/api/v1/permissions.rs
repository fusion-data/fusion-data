use axum::{
  Json,
  extract::{Path, State},
};
use fusion_core::application::Application;
use fusion_web::{WebResult, ok_json};
use modelsql::ModelManager;
use utoipa_axum::router::OpenApiRouter;

use hetuiam_core::types::{Permission, PermissionFilters, PermissionForCreate, PermissionForUpdate};

use crate::permission::PermissionSvc;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new().routes(utoipa_axum::routes!(
    create_permission,
    get_permission,
    update_permission,
    delete_permission,
    list_permissions
  ))
}

/// 创建权限
#[utoipa::path(
  post,
  path = "/permissions",
  request_body = PermissionForCreate,
  responses(
    (status = 201, description = "权限创建成功", body = i64),
    (status = 400, description = "请求参数错误")
  ),
  tag = "权限管理"
)]
async fn create_permission(State(app): State<Application>, Json(req): Json<PermissionForCreate>) -> WebResult<i64> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let permission_svc = PermissionSvc::new(mm);
  let id = permission_svc.create(req).await?;
  ok_json!(id)
}

/// 获取权限详情
#[utoipa::path(
  get,
  path = "/permissions/{id}",
  params(
    ("id" = i64, Path, description = "权限ID")
  ),
  responses(
    (status = 200, description = "获取成功", body = Option<Permission>),
    (status = 404, description = "权限不存在")
  ),
  tag = "权限管理"
)]
async fn get_permission(State(app): State<Application>, Path(id): Path<i64>) -> WebResult<Option<Permission>> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let permission_svc = PermissionSvc::new(mm);
  let permission = permission_svc.find_option_by_id(id).await?;
  ok_json!(permission)
}

/// 更新权限
#[utoipa::path(
  put,
  path = "/permissions/{id}",
  params(
    ("id" = i64, Path, description = "权限ID")
  ),
  request_body = PermissionForUpdate,
  responses(
    (status = 200, description = "更新成功"),
    (status = 404, description = "权限不存在")
  ),
  tag = "权限管理"
)]
async fn update_permission(
  State(app): State<Application>,
  Path(id): Path<i64>,
  Json(req): Json<PermissionForUpdate>,
) -> WebResult<()> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let permission_svc = PermissionSvc::new(mm);
  permission_svc.update_by_id(id, req).await?;
  ok_json!(())
}

/// 删除权限
#[utoipa::path(
  delete,
  path = "/permissions/{id}",
  params(
    ("id" = i64, Path, description = "权限ID")
  ),
  responses(
    (status = 200, description = "删除成功"),
    (status = 404, description = "权限不存在")
  ),
  tag = "权限管理"
)]
async fn delete_permission(State(app): State<Application>, Path(id): Path<i64>) -> WebResult<()> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let permission_svc = PermissionSvc::new(mm);
  permission_svc.delete_by_id(id).await?;
  ok_json!(())
}

/// 分页查询权限列表
#[utoipa::path(
  post,
  path = "/permissions/list",
  request_body = PermissionFilters,
  responses(
    (status = 200, description = "查询成功", body = modelsql::page::PageResult<Permission>),
    (status = 400, description = "请求参数错误")
  ),
  tag = "权限管理"
)]
async fn list_permissions(
  State(app): State<Application>,
  Json(req): Json<PermissionFilters>,
) -> WebResult<modelsql::page::PageResult<Permission>> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let permission_svc = PermissionSvc::new(mm);
  let result = permission_svc.page(req).await?;
  ok_json!(result)
}
