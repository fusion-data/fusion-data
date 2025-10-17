use axum::extract::FromRequestParts;
use fusion_web::WebError;
use fusionsql::{ModelManager, SqlError};
use jieyuan_core::model::{
  PathLookupRequest, PathLookupResponse, PathMappingEntity, PathMappingForCreateWithService, PathMappingForQuery,
  PathMappingForUpdate,
};
use std::collections::HashMap;
use std::time::Duration;

use crate::{
  model::{PathCacheBmc, PathMappingBmc},
  utils::model_manager_from_parts,
};
use jieyuan_core::model::PathParam;

/// 路径映射服务
#[derive(Clone)]
pub struct PathMappingSvc {
  mm: ModelManager,
}

impl PathMappingSvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  /// 获取 ModelManager 引用
  pub fn mm(&self) -> &ModelManager {
    &self.mm
  }

  /// 查找路径映射（带缓存）
  pub async fn lookup_path(&self, req: &PathLookupRequest) -> Result<Option<PathLookupResponse>, SqlError> {
    // 1. 尝试缓存查找
    let cache_key = format!("{}:{}:{}", req.service, req.method, req.path);
    if let Some(cached) = PathCacheBmc::get(&self.mm, &cache_key).await? {
      return Ok(Some(serde_json::from_value(cached)?));
    }

    // 2. 数据库查找
    if let Some(entity) = PathMappingBmc::find_by_path_pattern(&self.mm, &req.service, &req.method, &req.path).await? {
      // 3. 提取路径参数
      let params = Self::extract_path_params(&entity.path_pattern, &req.path)?;

      // 4. 构建响应
      let response = PathLookupResponse {
        action: entity.action,
        resource_tpl: entity.resource_tpl,
        path_params: params,
        cache_ttl: Some(300), // 5分钟缓存
      };

      // 5. 缓存结果
      PathCacheBmc::set(
        &self.mm,
        &cache_key,
        &req.service,
        &req.path,
        &req.method,
        &serde_json::to_value(&response).unwrap(),
        Duration::from_secs(300),
      )
      .await?;

      Ok(Some(response))
    } else {
      Ok(None)
    }
  }

  /// 提取路径参数
  pub fn extract_path_params(pattern: &str, actual: &str) -> Result<HashMap<String, String>, SqlError> {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let actual_parts: Vec<&str> = actual.split('/').collect();

    if pattern_parts.len() != actual_parts.len() {
      return Ok(HashMap::new());
    }

    let mut params = HashMap::new();

    for (pattern_part, actual_part) in pattern_parts.iter().zip(actual_parts.iter()) {
      if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
        let param_name = &pattern_part[1..pattern_part.len() - 1];
        params.insert(param_name.to_string(), actual_part.to_string());
      }
    }

    Ok(params)
  }

  /// 反序列化路径参数
  pub fn deserialize_path_params(entity: &PathMappingEntity) -> Result<Vec<PathParam>, SqlError> {
    serde_json::from_str(&entity.path_params)
      .map_err(|e| SqlError::InvalidArgument { message: format!("Invalid path_params JSON: {}", e) })
  }

  /// 创建路径映射
  pub async fn create_mapping(&self, request: PathMappingForCreateWithService) -> Result<PathMappingEntity, SqlError> {
    // 验证请求
    Self::validate_create_request(&request)?;

    // 转换 path_params 为 JSON 字符串
    let _path_params_json = serde_json::to_string(&request.path_params)
      .map_err(|e| SqlError::InvalidArgument { message: format!("Invalid path_params: {}", e) })?;

    // 创建映射请求
    let create_request: jieyuan_core::model::PathMappingForCreate = request.into();

    // 创建映射
    let id = PathMappingBmc::create(&self.mm, create_request).await?;

    // 获取创建的实体
    let entity = PathMappingBmc::get_by_id(&self.mm, id).await?.ok_or_else(|| SqlError::EntityNotFound {
      schema: None,
      entity: "service_path_mappings",
      id: id.into(),
    })?;

    // 清除相关缓存
    let _ = PathCacheBmc::clear_service_cache(&self.mm, &entity.service).await?;

    Ok(entity)
  }

  /// 更新路径映射
  pub async fn update_mapping(&self, id: i64, request: PathMappingForUpdate) -> Result<PathMappingEntity, SqlError> {
    // 验证请求
    Self::validate_update_request(&request)?;

    // 获取现有映射
    let existing = PathMappingBmc::get_by_id(&self.mm, id).await?.ok_or_else(|| SqlError::EntityNotFound {
      schema: None,
      entity: "service_path_mappings",
      id: id.into(),
    })?;

    // 更新映射
    PathMappingBmc::update_by_id(&self.mm, id, request).await?;

    // 获取更新后的实体
    let entity = PathMappingBmc::get_by_id(&self.mm, id).await?.ok_or_else(|| SqlError::EntityNotFound {
      schema: None,
      entity: "service_path_mappings",
      id: id.into(),
    })?;

    // 清除相关缓存
    PathCacheBmc::clear_service_cache(&self.mm, &existing.service).await?;

    Ok(entity)
  }

  /// 删除路径映射
  pub async fn delete_mapping(&self, id: i64) -> Result<(), SqlError> {
    let existing = PathMappingBmc::get_by_id(&self.mm, id).await?.ok_or_else(|| SqlError::EntityNotFound {
      schema: None,
      entity: "service_path_mappings",
      id: id.into(),
    })?;

    PathMappingBmc::delete_by_id(&self.mm, id).await?;

    // 清除相关缓存
    PathCacheBmc::clear_service_cache(&self.mm, &existing.service).await?;

    Ok(())
  }

  /// 列出路径映射
  pub async fn list_mappings(
    &self,
    query: PathMappingForQuery,
  ) -> Result<fusion_common::page::PageResult<PathMappingEntity>, SqlError> {
    PathMappingBmc::list_with_query(&self.mm, query).await
  }

  fn validate_create_request(request: &PathMappingForCreateWithService) -> Result<(), SqlError> {
    // 验证路径格式
    if !request.path_pattern.starts_with('/') {
      return Err(SqlError::InvalidArgument { message: "path_pattern must start with '/'".to_string() });
    }

    // 验证 HTTP 方法
    match request.method.as_str() {
      "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "*" => {}
      _ => return Err(SqlError::InvalidArgument { message: "invalid HTTP method".to_string() }),
    }

    // 验证动作格式
    if !request.action.contains(':') {
      return Err(SqlError::InvalidArgument { message: "action must be in format 'service:verb'".to_string() });
    }

    // 验证资源模板格式
    if !request.resource_tpl.starts_with("iam:") {
      return Err(SqlError::InvalidArgument { message: "resource_tpl must start with 'iam:'".to_string() });
    }

    Ok(())
  }

  fn validate_update_request(request: &PathMappingForUpdate) -> Result<(), SqlError> {
    if let Some(path_pattern) = &request.path_pattern
      && !path_pattern.starts_with('/')
    {
      return Err(SqlError::InvalidArgument { message: "path_pattern must start with '/'".to_string() });
    }

    if let Some(method) = &request.method {
      match method.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "*" => {}
        _ => return Err(SqlError::InvalidArgument { message: "invalid HTTP method".to_string() }),
      }
    }

    if let Some(action) = &request.action
      && !action.contains(':')
    {
      return Err(SqlError::InvalidArgument { message: "action must be in format 'service:verb'".to_string() });
    }

    if let Some(resource_tpl) = &request.resource_tpl
      && !resource_tpl.starts_with("iam:")
    {
      return Err(SqlError::InvalidArgument { message: "resource_tpl must start with 'iam:'".to_string() });
    }

    Ok(())
  }
}

impl FromRequestParts<fusion_core::application::Application> for PathMappingSvc {
  type Rejection = WebError;

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &fusion_core::application::Application,
  ) -> core::result::Result<Self, Self::Rejection> {
    let mm = model_manager_from_parts(parts, state)?;
    Ok(Self::new(mm))
  }
}
