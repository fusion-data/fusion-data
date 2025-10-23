use axum::extract::FromRequestParts;
use fusions::common::ahash::HashMap;
use fusions::common::page::PageResult;
use fusions::web::WebError;
use fusionsql::{ModelManager, SqlError};
use jieyuan_core::model::{
  IamResourceMappingEntity, IamResourceMappingForCreateWithService, IamResourceMappingForQuery,
  IamResourceMappingForUpdate, MappingParam, ResourceMappingLookupRequest, ResourceMappingLookupResponse,
};

use crate::access_control::resource_mapping_bmc::ResourceMappingBmc;

/// IAM 资源映射服务
#[derive(Clone)]
pub struct ResourceMappingSvc {
  mm: ModelManager,
}

impl ResourceMappingSvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  /// 获取 ModelManager 引用
  pub fn mm(&self) -> &ModelManager {
    &self.mm
  }

  /// 根据映射代码查找资源映射
  pub async fn lookup_by_code(&self, mapping_code: &str) -> Result<Option<ResourceMappingLookupResponse>, SqlError> {
    // 直接从数据库查找 - 不使用缓存
    if let Some(entity) = ResourceMappingBmc::find_by_code(&self.mm, mapping_code).await? {
      // 从 mapping_code 查找时，没有实际的路径参数提取
      // 但可以从 entity.mapping_params 字段获取预定义的参数
      // 注意：这里只能获取参数定义，实际值需要从 extras 中提供
      let params = Self::deserialize_mapping_params(&entity)?
        .into_iter()
        .filter_map(|param| param.default_value.clone().map(|value| (param.name, value)))
        .collect();

      // 构建响应
      let response = ResourceMappingLookupResponse {
        action: entity.action,
        resource_tpl: entity.resource_tpl,
        mapping_params: params,
      };

      Ok(Some(response))
    } else {
      Ok(None)
    }
  }

  /// 根据请求查找资源映射
  pub async fn lookup_path(
    &self,
    req: &ResourceMappingLookupRequest,
  ) -> Result<Option<ResourceMappingLookupResponse>, SqlError> {
    // 直接从数据库查找 - 不使用缓存
    if let Some(entity) = ResourceMappingBmc::find_by_path(&self.mm, &req.service, &req.path, &req.method).await? {
      // 提取路径参数
      let params = Self::extract_path_params(&entity.path_pattern, &req.path)?;

      // 构建响应
      let response = ResourceMappingLookupResponse {
        action: entity.action,
        resource_tpl: entity.resource_tpl,
        mapping_params: params,
      };

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
      return Ok(HashMap::default());
    }

    let mut params = HashMap::default();

    for (pattern_part, actual_part) in pattern_parts.iter().zip(actual_parts.iter()) {
      if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
        let param_name = &pattern_part[1..pattern_part.len() - 1];
        params.insert(param_name.to_string(), actual_part.to_string());
      }
    }

    Ok(params)
  }

  /// 反序列化映射参数
  pub fn deserialize_mapping_params(entity: &IamResourceMappingEntity) -> Result<Vec<MappingParam>, SqlError> {
    serde_json::from_str(&entity.mapping_params)
      .map_err(|e| SqlError::InvalidArgument { message: format!("Invalid mapping_params JSON: {}", e) })
  }

  /// 创建资源映射
  pub async fn create_mapping(
    &self,
    request: IamResourceMappingForCreateWithService,
  ) -> Result<IamResourceMappingEntity, SqlError> {
    // 验证请求
    Self::validate_create_request(&request)?;

    // 转换 mapping_params 为 JSON 字符串
    let _mapping_params_json = serde_json::to_string(&request.mapping_params)
      .map_err(|e| SqlError::InvalidArgument { message: format!("Invalid mapping_params: {}", e) })?;

    // 创建映射请求
    let create_request: jieyuan_core::model::IamResourceMappingForCreate = request.into();

    // 创建映射
    let id = ResourceMappingBmc::create(&self.mm, create_request).await?;

    // 获取创建的实体
    let entity = ResourceMappingBmc::get_by_id(&self.mm, id).await?.ok_or_else(|| SqlError::EntityNotFound {
      schema: None,
      entity: "iam_resource_mapping",
      id: id.into(),
    })?;

    Ok(entity)
  }

  /// 更新资源映射
  pub async fn update_mapping(
    &self,
    id: i64,
    request: IamResourceMappingForUpdate,
  ) -> Result<IamResourceMappingEntity, SqlError> {
    // 验证请求
    Self::validate_update_request(&request)?;

    // 获取现有映射
    let _existing = ResourceMappingBmc::get_by_id(&self.mm, id).await?.ok_or_else(|| SqlError::EntityNotFound {
      schema: None,
      entity: "iam_resource_mapping",
      id: id.into(),
    })?;

    // 更新映射
    ResourceMappingBmc::update_by_id(&self.mm, id, request).await?;

    // 获取更新后的实体
    let entity = ResourceMappingBmc::get_by_id(&self.mm, id).await?.ok_or_else(|| SqlError::EntityNotFound {
      schema: None,
      entity: "iam_resource_mapping",
      id: id.into(),
    })?;

    Ok(entity)
  }

  /// 删除资源映射
  pub async fn delete_mapping(&self, id: i64) -> Result<(), SqlError> {
    let _existing = ResourceMappingBmc::get_by_id(&self.mm, id).await?.ok_or_else(|| SqlError::EntityNotFound {
      schema: None,
      entity: "iam_resource_mapping",
      id: id.into(),
    })?;

    ResourceMappingBmc::delete_by_id(&self.mm, id).await?;

    Ok(())
  }

  /// 列出资源映射
  pub async fn list_mappings(
    &self,
    query: IamResourceMappingForQuery,
  ) -> Result<PageResult<IamResourceMappingEntity>, SqlError> {
    ResourceMappingBmc::list_with_query(&self.mm, query).await
  }

  fn validate_create_request(request: &IamResourceMappingForCreateWithService) -> Result<(), SqlError> {
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

  fn validate_update_request(request: &IamResourceMappingForUpdate) -> Result<(), SqlError> {
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

impl FromRequestParts<fusions::core::application::Application> for ResourceMappingSvc {
  type Rejection = WebError;

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &fusions::core::application::Application,
  ) -> core::result::Result<Self, Self::Rejection> {
    let mm = crate::utils::model_manager_from_parts(parts, state)?;
    Ok(Self::new(mm))
  }
}
