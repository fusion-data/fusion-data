use axum::{Json, extract::Path, http::StatusCode};
use fusion_common::model::IdI64Result;
use fusion_core::application::Application;
use fusion_web::{WebError, WebResult, ok_json};
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{User, UserForCreate, UserForPage, UserForUpdate};

use crate::UserSvc;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(user_create))
    .routes(utoipa_axum::routes!(user_get))
    .routes(utoipa_axum::routes!(user_update))
    .routes(utoipa_axum::routes!(user_delete))
    .routes(utoipa_axum::routes!(user_page))
}

/// 创建用户
#[utoipa::path(
  post,
  path = "/item",
  request_body = UserForCreate,
  responses(
    (status = 201, description = "用户创建成功", body = i64),
    (status = 400, description = "请求参数错误")
  ),
  tag = "用户管理"
)]
async fn user_create(
  user_svc: UserSvc,
  Json(req): Json<UserForCreate>,
) -> Result<(StatusCode, Json<IdI64Result>), WebError> {
  let id = user_svc.create(req).await?;
  Ok((StatusCode::CREATED, Json(IdI64Result::new(id))))
}

/// 获取用户详情
#[utoipa::path(
  get,
  path = "/item/{id}",
  params(
    ("id" = i64, Path, description = "用户ID")
  ),
  responses(
    (status = 200, description = "获取成功", body = Option<User>),
    (status = 404, description = "用户不存在")
  ),
  tag = "用户管理"
)]
async fn user_get(user_svc: UserSvc, Path(id): Path<i64>) -> WebResult<Option<User>> {
  let user = user_svc.find_option_by_id(id).await?;
  ok_json!(user)
}

/// 更新用户
#[utoipa::path(
  put,
  path = "/item/{id}",
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
async fn user_update(user_svc: UserSvc, Path(id): Path<i64>, Json(req): Json<UserForUpdate>) -> WebResult<()> {
  user_svc.update_by_id(id, req).await?;
  ok_json!(())
}

/// 删除用户
#[utoipa::path(
  delete,
  path = "/item/{id}",
  params(
    ("id" = i64, Path, description = "用户ID")
  ),
  responses(
    (status = 200, description = "删除成功"),
    (status = 404, description = "用户不存在")
  ),
  tag = "用户管理"
)]
async fn user_delete(user_svc: UserSvc, Path(id): Path<i64>) -> WebResult<()> {
  user_svc.delete_by_id(id).await?;
  ok_json!(())
}

/// 分页查询用户列表
#[utoipa::path(
  post,
  path = "/query",
  request_body = UserForPage,
  responses(
    (status = 200, description = "查询成功", body = fusionsql::page::PageResult<User>),
    (status = 400, description = "请求参数错误")
  ),
  tag = "用户管理"
)]
async fn user_page(user_svc: UserSvc, Json(req): Json<UserForPage>) -> WebResult<fusionsql::page::PageResult<User>> {
  let result = user_svc.page(req).await?;
  ok_json!(result)
}
