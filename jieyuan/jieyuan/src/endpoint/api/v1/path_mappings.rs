use axum::{Json, extract::Path, http::StatusCode};
use fusion_core::application::Application;
use fusion_web::{WebError, WebResult, ok_json};
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{
  PathMappingEntity, PathMappingForCreateWithService, PathMappingForQuery, PathMappingForUpdate,
};

use crate::service::PathMappingSvc;

/// 路径映射管理路由
pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list_mappings))
    .routes(utoipa_axum::routes!(create_mapping))
    .routes(utoipa_axum::routes!(get_mapping))
    .routes(utoipa_axum::routes!(update_mapping))
    .routes(utoipa_axum::routes!(delete_mapping))
    .routes(utoipa_axum::routes!(batch_operations))
}

/// 列出路径映射
#[utoipa::path(
  post,
  path = "/query",
  request_body = PathMappingForQuery,
  responses(
    (status = 200, description = "查询成功", body = PageResult<PathMappingEntity>),
    (status = 400, description = "请求参数错误")
  ),
  tag = "路径映射管理"
)]
pub async fn list_mappings(
  path_mapping_svc: PathMappingSvc,
  Json(query): Json<PathMappingForQuery>,
) -> WebResult<PageResult<PathMappingEntity>> {
  let result = path_mapping_svc.list_mappings(query).await.map_err(|e| WebError::bad_gateway(e.to_string()))?;
  ok_json!(result)
}

/// 创建路径映射
#[utoipa::path(
  post,
  path = "/",
  request_body = PathMappingForCreateWithService,
  responses(
    (status = 201, description = "路径映射创建成功", body = PathMappingEntity),
    (status = 400, description = "请求参数错误")
  ),
  tag = "路径映射管理"
)]
pub async fn create_mapping(
  path_mapping_svc: PathMappingSvc,
  Json(request): Json<PathMappingForCreateWithService>,
) -> Result<(StatusCode, Json<PathMappingEntity>), WebError> {
  let entity = path_mapping_svc.create_mapping(request).await.map_err(|e| WebError::bad_gateway(e.to_string()))?;
  Ok((StatusCode::CREATED, Json(entity)))
}

/// 获取单个路径映射
#[utoipa::path(
  get,
  path = "/{id}",
  params(
    ("id" = i64, Path, description = "路径映射ID")
  ),
  responses(
    (status = 200, description = "获取成功", body = PathMappingEntity),
    (status = 404, description = "路径映射不存在")
  ),
  tag = "路径映射管理"
)]
pub async fn get_mapping(path_mapping_svc: PathMappingSvc, Path(id): Path<i64>) -> WebResult<PathMappingEntity> {
  // We need to use the BMC directly since there's no get_by_id in the service
  let mm = path_mapping_svc.mm().clone();
  let entity = crate::model::PathMappingBmc::get_by_id(&mm, id)
    .await
    .map_err(|e| WebError::bad_gateway(e.to_string()))?
    .ok_or_else(|| WebError::new_with_code(404, "path mapping not found"))?;
  ok_json!(entity)
}

/// 更新路径映射
#[utoipa::path(
  put,
  path = "/{id}",
  params(
    ("id" = i64, Path, description = "路径映射ID")
  ),
  request_body = PathMappingForUpdate,
  responses(
    (status = 200, description = "更新成功", body = PathMappingEntity),
    (status = 400, description = "请求参数错误"),
    (status = 404, description = "路径映射不存在")
  ),
  tag = "路径映射管理"
)]
pub async fn update_mapping(
  path_mapping_svc: PathMappingSvc,
  Path(id): Path<i64>,
  Json(request): Json<PathMappingForUpdate>,
) -> WebResult<PathMappingEntity> {
  let entity = path_mapping_svc
    .update_mapping(id, request)
    .await
    .map_err(|e| WebError::bad_gateway(e.to_string()))?;
  ok_json!(entity)
}

/// 删除路径映射
#[utoipa::path(
  delete,
  path = "/{id}",
  params(
    ("id" = i64, Path, description = "路径映射ID")
  ),
  responses(
    (status = 200, description = "删除成功"),
    (status = 404, description = "路径映射不存在")
  ),
  tag = "路径映射管理"
)]
pub async fn delete_mapping(path_mapping_svc: PathMappingSvc, Path(id): Path<i64>) -> WebResult<()> {
  path_mapping_svc.delete_mapping(id).await.map_err(|e| WebError::bad_gateway(e.to_string()))?;
  ok_json!()
}

/// 批量操作
#[derive(serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct BatchOperation {
  /// 操作类型："enable"（启用）、"disable"（禁用）或"delete"（删除）
  pub action: String,
  /// 路径映射ID列表
  pub ids: Vec<i64>,
}

/// 批量操作路径映射
#[utoipa::path(
  post,
  path = "/batch",
  request_body = BatchOperation,
  responses(
    (status = 200, description = "批量操作成功", body = Vec<i64>),
    (status = 400, description = "请求参数错误")
  ),
  tag = "路径映射管理"
)]
pub async fn batch_operations(
  path_mapping_svc: PathMappingSvc,
  Json(request): Json<BatchOperation>,
) -> WebResult<Vec<i64>> {
  let mut results = Vec::new();

  for id in request.ids {
    let success = match request.action.as_str() {
      "enable" => {
        let update = PathMappingForUpdate { enabled: Some(true), ..Default::default() };
        path_mapping_svc.update_mapping(id, update).await.is_ok()
      }
      "disable" => {
        let update = PathMappingForUpdate { enabled: Some(false), ..Default::default() };
        path_mapping_svc.update_mapping(id, update).await.is_ok()
      }
      "delete" => path_mapping_svc.delete_mapping(id).await.is_ok(),
      _ => return Err(WebError::bad_request("invalid batch operation")),
    };

    if success {
      results.push(id);
    }
  }

  ok_json!(results)
}
