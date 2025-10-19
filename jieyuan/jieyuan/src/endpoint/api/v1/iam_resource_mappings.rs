use axum::{Json, extract::Path, http::StatusCode};
use fusion_core::application::Application;
use fusion_web::{WebError, WebResult, ok_json};
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{
  IamResourceMappingEntity, IamResourceMappingForCreateWithService, IamResourceMappingForQuery,
  IamResourceMappingForUpdate,
};

use crate::access_control::{
  ResourceMappingBmc, ResourceMappingCacheBmc, ResourceMappingLookupRequest, ResourceMappingLookupResponse,
  ResourceMappingSvc,
};

/// IAM 资源映射管理路由
pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list_mappings))
    .routes(utoipa_axum::routes!(create_mapping))
    .routes(utoipa_axum::routes!(get_mapping))
    .routes(utoipa_axum::routes!(update_mapping))
    .routes(utoipa_axum::routes!(delete_mapping))
    .routes(utoipa_axum::routes!(batch_operations))
    .routes(utoipa_axum::routes!(lookup_by_code))
    .routes(utoipa_axum::routes!(lookup_by_path))
    .routes(utoipa_axum::routes!(cache_operations))
}

/// 列出 IAM 资源映射
#[utoipa::path(
  post,
  path = "/query",
  request_body = IamResourceMappingForQuery,
  responses(
    (status = 200, description = "查询成功", body = fusion_common::page::PageResult<IamResourceMappingEntity>),
    (status = 400, description = "请求参数错误")
  ),
  tag = "IAM 资源映射管理"
)]
pub async fn list_mappings(
  mapping_svc: ResourceMappingSvc,
  Json(query): Json<IamResourceMappingForQuery>,
) -> WebResult<fusion_common::page::PageResult<IamResourceMappingEntity>> {
  let result = mapping_svc.list_mappings(query).await.map_err(|e| WebError::bad_gateway(e.to_string()))?;
  ok_json!(result)
}

/// 创建 IAM 资源映射
#[utoipa::path(
  post,
  path = "/",
  request_body = IamResourceMappingForCreateWithService,
  responses(
    (status = 201, description = "资源映射创建成功", body = IamResourceMappingEntity),
    (status = 400, description = "请求参数错误")
  ),
  tag = "IAM 资源映射管理"
)]
pub async fn create_mapping(
  mapping_svc: ResourceMappingSvc,
  Json(request): Json<IamResourceMappingForCreateWithService>,
) -> Result<(StatusCode, Json<IamResourceMappingEntity>), WebError> {
  let entity = mapping_svc.create_mapping(request).await.map_err(|e| WebError::bad_gateway(e.to_string()))?;
  Ok((StatusCode::CREATED, Json(entity)))
}

/// 获取单个 IAM 资源映射
#[utoipa::path(
  get,
  path = "/{id}",
  params(
    ("id" = i64, Path, description = "资源映射ID")
  ),
  responses(
    (status = 200, description = "获取成功", body = IamResourceMappingEntity),
    (status = 404, description = "资源映射不存在")
  ),
  tag = "IAM 资源映射管理"
)]
pub async fn get_mapping(mapping_svc: ResourceMappingSvc, Path(id): Path<i64>) -> WebResult<IamResourceMappingEntity> {
  // We need to use the BMC directly since there's no get_by_id in the service
  let mm = mapping_svc.mm().clone();
  let entity = ResourceMappingBmc::get_by_id(&mm, id)
    .await
    .map_err(|e| WebError::bad_gateway(e.to_string()))?
    .ok_or_else(|| WebError::new_with_code(404, "resource mapping not found"))?;
  ok_json!(entity)
}

/// 更新 IAM 资源映射
#[utoipa::path(
  put,
  path = "/{id}",
  params(
    ("id" = i64, Path, description = "资源映射ID")
  ),
  request_body = IamResourceMappingForUpdate,
  responses(
    (status = 200, description = "更新成功", body = IamResourceMappingEntity),
    (status = 400, description = "请求参数错误"),
    (status = 404, description = "资源映射不存在")
  ),
  tag = "IAM 资源映射管理"
)]
pub async fn update_mapping(
  mapping_svc: ResourceMappingSvc,
  Path(id): Path<i64>,
  Json(request): Json<IamResourceMappingForUpdate>,
) -> WebResult<IamResourceMappingEntity> {
  let entity = mapping_svc.update_mapping(id, request).await.map_err(|e| WebError::bad_gateway(e.to_string()))?;
  ok_json!(entity)
}

/// 删除 IAM 资源映射
#[utoipa::path(
  delete,
  path = "/{id}",
  params(
    ("id" = i64, Path, description = "资源映射ID")
  ),
  responses(
    (status = 200, description = "删除成功"),
    (status = 404, description = "资源映射不存在")
  ),
  tag = "IAM 资源映射管理"
)]
pub async fn delete_mapping(mapping_svc: ResourceMappingSvc, Path(id): Path<i64>) -> WebResult<()> {
  mapping_svc.delete_mapping(id).await.map_err(|e| WebError::bad_gateway(e.to_string()))?;
  ok_json!()
}

/// 批量操作
#[derive(serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct BatchOperation {
  /// 操作类型："enable"（启用）、"disable"（禁用）或"delete"（删除）
  pub action: String,
  /// 资源映射ID列表
  pub ids: Vec<i64>,
}

/// 批量操作 IAM 资源映射
#[utoipa::path(
  post,
  path = "/batch",
  request_body = BatchOperation,
  responses(
    (status = 200, description = "批量操作成功", body = Vec<i64>),
    (status = 400, description = "请求参数错误")
  ),
  tag = "IAM 资源映射管理"
)]
pub async fn batch_operations(
  mapping_svc: ResourceMappingSvc,
  Json(request): Json<BatchOperation>,
) -> WebResult<Vec<i64>> {
  let mut results = Vec::new();

  for id in request.ids {
    let success = match request.action.as_str() {
      "enable" => {
        let update = IamResourceMappingForUpdate { enabled: Some(true), ..Default::default() };
        mapping_svc.update_mapping(id, update).await.is_ok()
      }
      "disable" => {
        let update = IamResourceMappingForUpdate { enabled: Some(false), ..Default::default() };
        mapping_svc.update_mapping(id, update).await.is_ok()
      }
      "delete" => mapping_svc.delete_mapping(id).await.is_ok(),
      _ => return Err(WebError::bad_request("invalid batch operation")),
    };

    if success {
      results.push(id);
    }
  }

  ok_json!(results)
}

/// 根据映射代码查找
#[utoipa::path(
  get,
  path = "/lookup/code/{mapping_code}",
  params(
    ("mapping_code" = String, Path, description = "映射代码")
  ),
  responses(
    (status = 200, description = "查找成功", body = ResourceMappingLookupResponse),
    (status = 404, description = "映射不存在")
  ),
  tag = "IAM 资源映射管理"
)]
pub async fn lookup_by_code(
  mapping_svc: ResourceMappingSvc,
  Path(mapping_code): Path<String>,
) -> WebResult<ResourceMappingLookupResponse> {
  let response = mapping_svc
    .lookup_by_code(&mapping_code)
    .await
    .map_err(|e| WebError::bad_gateway(e.to_string()))?
    .ok_or_else(|| WebError::new_with_code(404, "mapping not found"))?;
  ok_json!(response)
}

/// 根据路径查找
#[utoipa::path(
  post,
  path = "/lookup/path",
  request_body = ResourceMappingLookupRequest,
  responses(
    (status = 200, description = "查找成功", body = ResourceMappingLookupResponse),
    (status = 404, description = "映射不存在")
  ),
  tag = "IAM 资源映射管理"
)]
pub async fn lookup_by_path(
  mapping_svc: ResourceMappingSvc,
  Json(request): Json<ResourceMappingLookupRequest>,
) -> WebResult<ResourceMappingLookupResponse> {
  let response = mapping_svc
    .lookup_path(&request)
    .await
    .map_err(|e| WebError::bad_gateway(e.to_string()))?
    .ok_or_else(|| WebError::new_with_code(404, "mapping not found"))?;
  ok_json!(response)
}

/// 缓存操作
#[derive(serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct CacheOperation {
  /// 操作类型："clear"（清除）、"cleanup"（清理过期）、"stats"（统计）
  pub action: String,
  /// 服务名称（用于清除特定服务的缓存）
  pub service: Option<String>,
}

/// 缓存操作
#[utoipa::path(
  post,
  path = "/cache",
  request_body = CacheOperation,
  responses(
    (status = 200, description = "缓存操作成功"),
    (status = 400, description = "请求参数错误")
  ),
  tag = "IAM 资源映射管理"
)]
pub async fn cache_operations(
  mapping_svc: ResourceMappingSvc,
  Json(request): Json<CacheOperation>,
) -> WebResult<serde_json::Value> {
  let mm = mapping_svc.mm().clone();

  match request.action.as_str() {
    "clear" => {
      if let Some(service) = request.service {
        // 清除特定服务的缓存
        let count = ResourceMappingBmc::clear_service_cache(&mm, &service)
          .await
          .map_err(|e| WebError::bad_gateway(e.to_string()))?;
        ok_json!(serde_json::json!({ "cleared": count, "service": service }))
      } else {
        // 清除所有缓存
        let count = ResourceMappingCacheBmc::clear_by_pattern(&mm, "%")
          .await
          .map_err(|e| WebError::bad_gateway(e.to_string()))?;
        ok_json!(serde_json::json!({ "cleared": count }))
      }
    }
    "cleanup" => {
      // 清理过期缓存
      let count = ResourceMappingCacheBmc::cleanup_expired(&mm)
        .await
        .map_err(|e| WebError::bad_gateway(e.to_string()))?;
      ok_json!(serde_json::json!({ "cleaned": count }))
    }
    "stats" => {
      // 获取缓存统计
      let stats = ResourceMappingCacheBmc::get_cache_stats(&mm)
        .await
        .map_err(|e| WebError::bad_gateway(e.to_string()))?;
      ok_json!(serde_json::json!(stats))
    }
    _ => Err(WebError::bad_request("invalid cache operation")),
  }
}
