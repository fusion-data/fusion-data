use chrono::{DateTime, Utc};
use fusion_common::{ahash::HashMap, page::Page};
use fusionsql_core::filter::{OpValBool, OpValInt64, OpValString};
use serde::{Deserialize, Serialize};

/// IAM 资源映射实体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct IamResourceMappingEntity {
  pub id: i64,
  pub service: String,
  pub path_pattern: String,
  pub method: String,
  pub action: String,
  pub resource_tpl: String,
  pub mapping_params: String, // JSON string of MappingParam array
  pub enabled: bool,
  pub tenant_id: Option<i64>, // Added tenant isolation support
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub created_by: i64,
  pub updated_by: Option<i64>,
  pub description: Option<String>,
  pub mapping_code: Option<String>, // Added mapping code support
}

/// 映射参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct MappingParam {
  pub name: String,
  pub param_type: String, // "uuid", "i64", "string"
  pub required: bool,
  pub default_value: Option<String>,
  pub description: Option<String>,
}

/// 创建 IAM 资源映射请求
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct IamResourceMappingForCreate {
  pub path_pattern: String,
  pub method: String,
  pub action: String,
  pub resource_tpl: String,
  pub mapping_params: String, // JSON string of MappingParam array
  pub enabled: Option<bool>,
  pub description: Option<String>,
  pub tenant_id: Option<i64>,       // Added tenant support
  pub mapping_code: Option<String>, // Added mapping code support
}

/// 插入 IAM 资源映射请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct IamResourceMappingForInsert {
  pub service: String,
  pub path_pattern: String,
  pub method: String,
  pub action: String,
  pub resource_tpl: String,
  pub mapping_params: String, // JSON string of MappingParam array
  pub enabled: bool,
  pub description: Option<String>,
  pub tenant_id: Option<i64>,       // Added tenant support
  pub mapping_code: Option<String>, // Added mapping code support
}

/// 创建 IAM 资源映射请求（带服务名）
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct IamResourceMappingForCreateWithService {
  pub service: String,
  pub path_pattern: String,
  pub method: String,
  pub action: String,
  pub resource_tpl: String,
  pub mapping_params: String, // JSON string of MappingParam array
  pub enabled: Option<bool>,
  pub description: Option<String>,
  pub tenant_id: Option<i64>,       // Added tenant support
  pub mapping_code: Option<String>, // Added mapping code support
}

impl From<IamResourceMappingForCreateWithService> for IamResourceMappingForInsert {
  fn from(value: IamResourceMappingForCreateWithService) -> Self {
    Self {
      service: value.service,
      path_pattern: value.path_pattern,
      method: value.method,
      action: value.action,
      resource_tpl: value.resource_tpl,
      mapping_params: value.mapping_params,
      enabled: value.enabled.unwrap_or(true),
      description: value.description,
      tenant_id: value.tenant_id,
      mapping_code: value.mapping_code,
    }
  }
}

impl From<IamResourceMappingForCreateWithService> for IamResourceMappingForCreate {
  fn from(value: IamResourceMappingForCreateWithService) -> Self {
    Self {
      path_pattern: value.path_pattern,
      method: value.method,
      action: value.action,
      resource_tpl: value.resource_tpl,
      mapping_params: value.mapping_params,
      enabled: value.enabled,
      description: value.description,
      tenant_id: value.tenant_id,
      mapping_code: value.mapping_code,
    }
  }
}

/// 更新 IAM 资源映射请求
#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct IamResourceMappingForUpdate {
  pub path_pattern: Option<String>,
  pub method: Option<String>,
  pub action: Option<String>,
  pub resource_tpl: Option<String>,
  pub mapping_params: Option<String>, // JSON string of MappingParam array
  pub enabled: Option<bool>,
  pub description: Option<String>,
  pub tenant_id: Option<i64>,       // Added tenant support
  pub mapping_code: Option<String>, // Added mapping code support
}

/// IAM 资源映射查询过滤器
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct IamResourceMappingFilter {
  pub service: Option<OpValString>,
  pub method: Option<OpValString>,
  pub enabled: Option<OpValBool>,
  pub tenant_id: Option<OpValInt64>,     // Added tenant filter
  pub mapping_code: Option<OpValString>, // Added mapping code filter
  pub search: Option<OpValString>,
  pub created_by: Option<OpValInt64>,
}

/// IAM 资源映射查询
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct IamResourceMappingForQuery {
  #[serde(default)]
  pub page: Page,
  #[serde(default)]
  pub filters: Vec<IamResourceMappingFilter>,
}

/// 表名常量
pub const TABLE_IAM_RESOURCE_MAPPING: &str = "iam_resource_mapping";

/// 资源映射查找请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct ResourceMappingLookupRequest {
  pub service: String,
  pub path: String,
  pub method: String,
}

/// 资源映射查找响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct ResourceMappingLookupResponse {
  pub action: String,
  pub resource_tpl: String,
  pub mapping_params: HashMap<String, String>,
}

/// 解析后的资源映射
#[derive(Debug, Clone)]
pub struct ResolvedResourceMapping {
  pub action: String,
  pub resource_tpl: String,
  pub extracted_params: HashMap<String, String>,
}
