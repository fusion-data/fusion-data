use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 路径查找请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathLookupRequest {
  pub service: String,
  pub path: String,
  pub method: String,
}

/// 路径查找响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathLookupResponse {
  pub action: String,
  pub resource_tpl: String,
  pub path_params: HashMap<String, String>,
  pub cache_ttl: Option<u64>,
}

/// 解析后的路径映射
#[derive(Debug, Clone)]
pub struct ResolvedPathMapping {
  pub action: String,
  pub resource_tpl: String,
  pub extracted_params: HashMap<String, String>,
}
