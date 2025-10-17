use fusion_common::ctx::Ctx;
use fusion_core::DataError;
use fusion_web::WebError;
use serde::Serialize;

use crate::model::path_authz::MatchedMapping;

/// Jieyuan 客户端扩展
#[derive(Clone)]
#[allow(dead_code)]
pub struct JieyuanClient {
  base_url: String,
  timeout_ms: u64,
  client: reqwest::Client,
}

impl JieyuanClient {
  pub fn new() -> Result<Self, DataError> {
    let base_url = "http://localhost:50010".to_string();
    let timeout_ms = 5000;

    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_millis(timeout_ms))
      .build()
      .map_err(|e| DataError::server_error(format!("failed to create HTTP client: {}", e)))?;

    Ok(Self { base_url, timeout_ms, client })
  }

  /// 基于路径的权限检查
  pub async fn authorize_by_path(
    &self,
    _token: &str,
    _service: &str,
    _path: &str,
    _method: &str,
    request_ip: &str,
  ) -> Result<PathAuthzResponse, WebError> {
    // 简化实现，直接返回允许的响应
    // 在实际项目中，这里应该调用 jieyuan API
    todo!()
  }
}

/// 基于路径的授权响应
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PathAuthzResponse {
  pub decision: String,
  pub ctx: Option<Ctx>,
  pub matched_mapping: Option<MatchedMapping>,
}
