use chrono::{DateTime, Utc};
use fusionsql_core::filter::{OpValBool, OpValInt64, OpValString};
use serde::{Deserialize, Serialize};

/// 路径映射实体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathMappingEntity {
  pub id: i64,
  pub service: String,
  pub path_pattern: String,
  pub method: String,
  pub action: String,
  pub resource_tpl: String,
  pub path_params: String, // JSON string of PathParam array
  pub enabled: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub created_by: i64,
  pub updated_by: Option<i64>,
  pub description: Option<String>,
}

/// 路径参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathParam {
  pub name: String,
  pub param_type: String, // "uuid", "i64", "string"
  pub required: bool,
  pub default_value: Option<String>,
  pub description: Option<String>,
}

/// 创建路径映射请求
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathMappingForCreate {
  pub path_pattern: String,
  pub method: String,
  pub action: String,
  pub resource_tpl: String,
  pub path_params: String, // JSON string of PathParam array
  pub enabled: Option<bool>,
  pub description: Option<String>,
}

/// 插入路径映射请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathMappingForInsert {
  pub service: String,
  pub path_pattern: String,
  pub method: String,
  pub action: String,
  pub resource_tpl: String,
  pub path_params: String, // JSON string of PathParam array
  pub enabled: bool,
  pub description: Option<String>,
}

/// 创建路径映射请求（带服务名）
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathMappingForCreateWithService {
  pub service: String,
  pub path_pattern: String,
  pub method: String,
  pub action: String,
  pub resource_tpl: String,
  pub path_params: String, // JSON string of PathParam array
  pub enabled: Option<bool>,
  pub description: Option<String>,
}

impl From<PathMappingForCreateWithService> for PathMappingForInsert {
  fn from(value: PathMappingForCreateWithService) -> Self {
    Self {
      service: value.service,
      path_pattern: value.path_pattern,
      method: value.method,
      action: value.action,
      resource_tpl: value.resource_tpl,
      path_params: value.path_params,
      enabled: value.enabled.unwrap_or(true),
      description: value.description,
    }
  }
}

impl From<PathMappingForCreateWithService> for PathMappingForCreate {
  fn from(value: PathMappingForCreateWithService) -> Self {
    Self {
      path_pattern: value.path_pattern,
      method: value.method,
      action: value.action,
      resource_tpl: value.resource_tpl,
      path_params: value.path_params,
      enabled: value.enabled,
      description: value.description,
    }
  }
}

/// 更新路径映射请求
#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathMappingForUpdate {
  pub path_pattern: Option<String>,
  pub method: Option<String>,
  pub action: Option<String>,
  pub resource_tpl: Option<String>,
  pub path_params: Option<String>, // JSON string of PathParam array
  pub enabled: Option<bool>,
  pub description: Option<String>,
}

/// 路径映射查询过滤器
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathMappingFilter {
  pub service: Option<OpValString>,
  pub method: Option<OpValString>,
  pub enabled: Option<OpValBool>,
  pub search: Option<OpValString>,
  pub created_by: Option<OpValInt64>,
}

/// 路径映射查询
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathMappingForQuery {
  #[serde(default)]
  pub page: fusion_common::page::Page,
  #[serde(default)]
  pub filters: Vec<PathMappingFilter>,
}

/// 表名常量
pub const TABLE_PATH_MAPPING: &str = "service_path_mappings";
