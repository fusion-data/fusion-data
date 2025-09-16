use axum::{
  Json,
  extract::{Path, State},
};
use fusion_core::application::Application;
use fusion_web::{WebResult, ok_json};
use modelsql::{ModelManager, page::PageResult};
use utoipa_axum::router::OpenApiRouter;

use hetuiam_core::types::{CreateRoleDto, Role, RoleFilters, RoleForUpdate};

use crate::{ctx_w::CtxW, role::RoleSvc};

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new().routes(utoipa_axum::routes!(create_role, get_role, update_role, delete_role, list_roles))
}

/// 创建角色
#[utoipa::path(
  post,
  path = "/roles",
  request_body = CreateRoleDto,
  responses(
    (status = 201, description = "角色创建成功", body = i64),
    (status = 400, description = "请求参数错误")
  ),
  tag = "角色管理"
)]
async fn create_role(State(app): State<Application>, Json(req): Json<CreateRoleDto>) -> WebResult<i64> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let role_svc = RoleSvc::new(mm);
  let id = role_svc.create(req).await?;
  ok_json!(id)
}

/// 获取角色详情
#[utoipa::path(
  get,
  path = "/roles/{id}",
  params(
    ("id" = i64, Path, description = "角色ID")
  ),
  responses(
    (status = 200, description = "获取成功", body = Option<Role>),
    (status = 404, description = "角色不存在")
  ),
  tag = "角色管理"
)]
async fn get_role(State(app): State<Application>, Path(id): Path<i64>) -> WebResult<Option<Role>> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let role_svc = RoleSvc::new(mm);
  let role = role_svc.find_option_by_id(id).await?;
  ok_json!(role)
}

/// 更新角色
#[utoipa::path(
  put,
  path = "/roles/{id}",
  params(
    ("id" = i64, Path, description = "角色ID")
  ),
  request_body = RoleForUpdate,
  responses(
    (status = 200, description = "更新成功"),
    (status = 404, description = "角色不存在")
  ),
  tag = "角色管理"
)]
async fn update_role(
  State(app): State<Application>,
  Path(id): Path<i64>,
  Json(req): Json<RoleForUpdate>,
) -> WebResult<()> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let role_svc = RoleSvc::new(mm);
  role_svc.update_by_id(id, req).await?;
  ok_json!(())
}

/// 删除角色
#[utoipa::path(
  delete,
  path = "/roles/{id}",
  params(
    ("id" = i64, Path, description = "角色ID")
  ),
  responses(
    (status = 200, description = "删除成功"),
    (status = 404, description = "角色不存在")
  ),
  tag = "角色管理"
)]
async fn delete_role(State(app): State<Application>, Path(id): Path<i64>) -> WebResult<()> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let role_svc = RoleSvc::new(mm);
  role_svc.delete_by_id(id).await?;
  ok_json!(())
}

/// 分页查询角色列表
#[utoipa::path(
  post,
  path = "/roles/list",
  request_body = RoleFilters,
  responses(
    (status = 200, description = "查询成功", body = modelsql::page::PageResult<Role>),
    (status = 400, description = "请求参数错误")
  ),
  tag = "角色管理"
)]
async fn list_roles(State(app): State<Application>, Json(req): Json<RoleFilters>) -> WebResult<PageResult<Role>> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let role_svc = RoleSvc::new(mm);
  let result = role_svc.page(req).await?;
  ok_json!(result)
}
