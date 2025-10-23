use axum::{
  Json,
  extract::{Path, State},
};
use fusions::core::application::Application;
use fusions::web::{WebResult, ok_json};
use fusionsql::{ModelManager, page::PageResult};
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{PolicyEntity, PolicyForCreate, PolicyForPage, PolicyForUpdate};

use crate::access_control::PolicySvc;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(create_policy))
    .routes(utoipa_axum::routes!(get_policy))
    .routes(utoipa_axum::routes!(update_policy))
    .routes(utoipa_axum::routes!(delete_policy))
    .routes(utoipa_axum::routes!(list_policies))
}

/// 创建策略
#[utoipa::path(
  post,
  path = "/item",
  request_body = PolicyForCreate,
  responses(
    (status = 201, description = "策略创建成功", body = i64),
    (status = 400, description = "请求参数错误")
  ),
  tag = "策略管理"
)]
async fn create_policy(State(app): State<Application>, Json(req): Json<PolicyForCreate>) -> WebResult<i64> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let policy_svc = PolicySvc::new(mm);
  let id = policy_svc.create(req).await?;
  Ok(Json(id))
}

/// 获取策略详情
#[utoipa::path(
  get,
  path = "/item/{id}",
  params(
    ("id" = i64, Path, description = "策略ID")
  ),
  responses(
    (status = 200, description = "获取成功", body = Option<PolicyEntity>),
    (status = 404, description = "策略不存在")
  ),
  tag = "策略管理"
)]
async fn get_policy(State(app): State<Application>, Path(id): Path<i64>) -> WebResult<Option<PolicyEntity>> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let policy_svc = PolicySvc::new(mm);
  let policy = policy_svc.find_option_by_id(id).await?;
  ok_json!(policy)
}

/// 更新策略
#[utoipa::path(
  put,
  path = "/item/{id}",
  params(
    ("id" = i64, Path, description = "策略ID")
  ),
  request_body = PolicyForUpdate,
  responses(
    (status = 200, description = "更新成功"),
    (status = 404, description = "策略不存在")
  ),
  tag = "策略管理"
)]
async fn update_policy(
  State(app): State<Application>,
  Path(id): Path<i64>,
  Json(req): Json<PolicyForUpdate>,
) -> WebResult<()> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let policy_svc = PolicySvc::new(mm);
  policy_svc.update_by_id(id, req).await?;
  ok_json!()
}

/// 删除策略
#[utoipa::path(
  delete,
  path = "/item/{id}",
  params(
    ("id" = i64, Path, description = "策略ID")
  ),
  responses(
    (status = 200, description = "删除成功"),
    (status = 404, description = "策略不存在")
  ),
  tag = "策略管理"
)]
async fn delete_policy(State(app): State<Application>, Path(id): Path<i64>) -> WebResult<()> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let policy_svc = PolicySvc::new(mm);
  policy_svc.delete_by_id(id).await?;
  ok_json!()
}

/// 查询策略列表
#[utoipa::path(
  post,
  path = "/page",
  request_body = PolicyForPage,
  responses(
    (status = 200, description = "查询成功", body = fusionsql::page::PageResult<PolicyEntity>),
    (status = 400, description = "请求参数错误")
  ),
  tag = "策略管理"
)]
async fn list_policies(
  State(app): State<Application>,
  Json(req): Json<PolicyForPage>,
) -> WebResult<PageResult<PolicyEntity>> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let policy_svc = PolicySvc::new(mm);
  let result = policy_svc.page(req).await?;
  ok_json!(result)
}
