use fusion_common::ctx::Ctx;
use fusion_core::DataError;
use fusion_web::WebError;
use serde::Serialize;

use crate::model::{AuthorizeRequest, AuthorizeResponse, path_authz::MatchedMapping};

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

  /// 基于资源映射的权限检查（支持 mapping_code 和传统路径映射）
  ///
  /// 所有权限检查都必须通过配置的资源映射表获取 action 和 resource_tpl，
  /// 确保授权规则的一致性和统一管理。
  ///
  /// # 参数
  /// - `token`: 认证令牌
  /// - `service`: 服务名称（必需，除非使用 mapping_code）
  /// - `path`: 请求路径（必需，除非使用 mapping_code）
  /// - `method`: HTTP 方法（必需）
  /// - `request_ip`: 客户端 IP（可选）
  pub async fn authorize(
    &self,
    token: &str,
    authorize_request: AuthorizeRequest,
  ) -> Result<ResourceMappingAuthzResponse, WebError> {
    // 发送 HTTP 请求到 authorize API
    let url = format!("{}/api/v1/iam/authorize", self.base_url);

    let response = self
      .client
      .post(&url)
      .bearer_auth(token)
      .json(&authorize_request)
      .send()
      .await
      .map_err(|e| WebError::new_with_code(500, format!("failed to send authorization request: {}", e)))?;

    // 处理响应
    if response.status().is_success() {
      // 直接反序列化 AuthorizeResponse（现在支持 Ctx 反序列化）
      let authorize_response: AuthorizeResponse = response
        .json()
        .await
        .map_err(|e| WebError::new_with_code(500, format!("failed to parse authorization response: {}", e)))?;

      // 构造成功响应
      Ok(ResourceMappingAuthzResponse {
        decision: authorize_response.decision.to_string(),
        ctx: Some(authorize_response.ctx),
        matched_mapping: None, // 资源映射信息由调用方处理
      })
    } else {
      // 处理错误响应
      let status = response.status();
      let error_text = response.text().await.unwrap_or_else(|_| "Failed to read error response".to_string());

      if status.as_u16() == 401 {
        Err(WebError::new_with_code(401, "unauthorized: invalid token"))
      } else if status.as_u16() == 403 {
        Err(WebError::new_with_code(403, format!("access denied: {}", error_text)))
      } else {
        Err(WebError::new_with_code(status.as_u16() as i32, format!("authorization request failed: {}", error_text)))
      }
    }
  }
}

/// 基于资源映射的授权响应
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ResourceMappingAuthzResponse {
  pub decision: String,
  pub ctx: Option<Ctx>,
  pub matched_mapping: Option<MatchedMapping>,
}
