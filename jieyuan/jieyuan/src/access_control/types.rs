use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 资源映射查找请求
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ResourceMappingLookupRequest {
  pub service: String,
  pub path: String,
  pub method: String,
}

/// 资源映射查找响应
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ResourceMappingLookupResponse {
  pub action: String,
  pub resource_tpl: String,
  pub mapping_params: HashMap<String, String>,
  pub cache_ttl: Option<u64>,
}

/// 解析后的资源映射
#[derive(Debug, Clone)]
pub struct ResolvedResourceMapping {
  pub action: String,
  pub resource_tpl: String,
  pub extracted_params: HashMap<String, String>,
}
