use axum::{
  Json,
  extract::{Path, State},
};
use fusion_core::{IdI64Result, application::Application};
use fusion_web::{WebResult, ok_json};
use modelsql::ModelManager;
use utoipa_axum::router::OpenApiRouter;

use hetuiam_core::types::{User, UserForCreate, UserForPage, UserForUpdate};

use crate::{UserSvc, ctx_w::CtxW};

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new().routes(utoipa_axum::routes!(create_user, get_user, update_user, delete_user, list_users))
}

/// 创建用户
#[utoipa::path(
  post,
  path = "/users",
  request_body = UserForCreate,
  responses(
    (status = 201, description = "用户创建成功", body = i64),
    (status = 400, description = "请求参数错误")
  ),
  tag = "用户管理"
)]
async fn create_user(State(app): State<Application>, Json(req): Json<UserForCreate>) -> WebResult<IdI64Result> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm);
  let id = user_svc.create(req).await?;
  ok_json!(IdI64Result::new(id))
}

/// 获取用户详情
#[utoipa::path(
  get,
  path = "/users/{id}",
  params(
    ("id" = i64, Path, description = "用户ID")
  ),
  responses(
    (status = 200, description = "获取成功", body = Option<User>),
    (status = 404, description = "用户不存在")
  ),
  tag = "用户管理"
)]
async fn get_user(State(app): State<Application>, Path(id): Path<i64>) -> WebResult<Option<User>> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm);
  let user = user_svc.find_option_by_id(id).await?;
  ok_json!(user)
}

/// 更新用户
#[utoipa::path(
  put,
  path = "/users/{id}",
  params(
    ("id" = i64, Path, description = "用户ID")
  ),
  request_body = UserForUpdate,
  responses(
    (status = 200, description = "更新成功"),
    (status = 404, description = "用户不存在")
  ),
  tag = "用户管理"
)]
async fn update_user(
  State(app): State<Application>,
  Path(id): Path<i64>,
  Json(req): Json<UserForUpdate>,
) -> WebResult<()> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm);
  user_svc.update_by_id(id, req).await?;
  ok_json!(())
}

/// 删除用户
#[utoipa::path(
  delete,
  path = "/users/{id}",
  params(
    ("id" = i64, Path, description = "用户ID")
  ),
  responses(
    (status = 200, description = "删除成功"),
    (status = 404, description = "用户不存在")
  ),
  tag = "用户管理"
)]
async fn delete_user(State(app): State<Application>, Path(id): Path<i64>) -> WebResult<()> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm);
  user_svc.delete_by_id(id).await?;
  ok_json!(())
}

/// 分页查询用户列表
#[utoipa::path(
  post,
  path = "/users/list",
  request_body = UserForPage,
  responses(
    (status = 200, description = "查询成功", body = modelsql::page::PageResult<User>),
    (status = 400, description = "请求参数错误")
  ),
  tag = "用户管理"
)]
async fn list_users(
  State(app): State<Application>,
  Json(req): Json<UserForPage>,
) -> WebResult<modelsql::page::PageResult<User>> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm);
  let result = user_svc.page(req).await?;
  ok_json!(result)
}
