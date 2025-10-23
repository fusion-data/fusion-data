use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 基于路径的授权请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathBasedAuthzRequest {
  /// 服务名称
  pub service: String,
  /// 请求路径
  pub path: String,
  /// HTTP 方法
  pub method: String,
  /// 客户端 IP 地址
  pub request_ip: Option<String>,
}

/// 匹配的路径映射信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct MatchedMapping {
  /// 授权动作
  pub action: String,
  /// 资源模板
  pub resource_tpl: String,
  /// 提取的路径参数
  pub extracted_params: HashMap<String, String>,
}

/// 基于路径的授权响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathBasedAuthzResponse {
  /// 授权决策结果："allow" 或 "deny"
  pub decision: String,
  /// 用户上下文信息
  pub ctx: Option<PathAuthzCtxPayload>,
  /// 匹配的路径映射信息
  pub matched_mapping: Option<MatchedMapping>,
}

/// 路径授权专用的上下文载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathAuthzCtxPayload {
  pub tenant_id: i64,
  pub sub: i64,
  pub roles: Vec<String>,
  pub is_platform_admin: bool,
  pub token_seq: i32,
  pub method: String,
  pub path: String,
  pub request_ip: String,
  pub req_time: String,
}
