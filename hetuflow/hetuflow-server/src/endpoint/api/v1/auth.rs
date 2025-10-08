//! JWE Token 认证 API 端点
//!
//! 提供 JWE Token 生成接口，仅允许本机访问

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use axum::{
  extract::{ConnectInfo, State},
  http::StatusCode,
  response::Json,
};
use chrono::Utc;
use fusion_web::WebError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa_axum::router::OpenApiRouter;

use crate::{
  application::ServerApplication,
  service::{JweError, JweSvc},
};

/// 认证相关路由
pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new().routes(utoipa_axum::routes!(generate_token))
}

/// 生成 Token 请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GenerateTokenRequest {
  /// Agent ID (必填，UUID 格式)
  pub agent_id: String,
  /// 权限列表 (可选)
  pub permissions: Option<Vec<String>>,
}

/// 生成 Token 响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GenerateTokenResponse {
  /// JWE Token
  pub token: String,
  /// Agent ID
  pub agent_id: String,
  /// Token 类型
  pub token_type: String,
  /// 过期时间 (Unix 时间戳)
  pub expires_at: i64,
  /// 签发时间 (ISO 8601 格式)
  pub issued_at: String,
}

/// 检查是否为本机访问
fn is_localhost(addr: &SocketAddr) -> bool {
  match addr.ip() {
    IpAddr::V4(ipv4) => ipv4 == Ipv4Addr::LOCALHOST,
    IpAddr::V6(ipv6) => ipv6 == Ipv6Addr::LOCALHOST,
  }
}

/// 生成 JWE Token (仅本机访问)
#[utoipa::path(
  post,
  path = "/generate-token",
  request_body = GenerateTokenRequest,
  responses(
    (status = 200, description = "Success", body = GenerateTokenResponse),
    (status = 403, description = "Access Denied", body = WebError),
    (status = 500, description = "Internal Server Error", body = WebError)
  )
)]
#[axum::debug_handler]
pub async fn generate_token(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  State(app): State<ServerApplication>,
  Json(request): Json<GenerateTokenRequest>,
) -> Result<Json<GenerateTokenResponse>, (StatusCode, Json<WebError>)> {
  // 检查是否为本机访问
  if !is_localhost(&addr) {
    return Err((
      StatusCode::FORBIDDEN,
      Json(WebError::new(
        403,
        "This API endpoint is only accessible from localhost",
        Some(Box::new(json!({"remote_addr": addr.to_string()}))),
      )),
    ));
  }

  // 解析 agent_id
  let agent_id = &request.agent_id;

  // 获取 JWE 配置
  let jwe_config =
    app.setting().jwe.as_ref().ok_or_else(|| {
      (StatusCode::INTERNAL_SERVER_ERROR, Json(WebError::new(500, "JWE service not configured", None)))
    })?;

  // 创建 JWE 服务
  let jwe_service = JweSvc::new(jwe_config.clone()).map_err(|e| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(WebError::new(500, "JWE service initialization failed", Some(Box::new(json!({"error": e.to_string()}))))),
    )
  })?;

  // 生成 Token
  let permissions = request.permissions.unwrap_or_default();
  let server_id = &app.setting().server.server_id;

  let token = jwe_service.generate_token(agent_id, server_id, permissions).map_err(|e| match e {
    JweError::TokenGenerationFailed(msg) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(WebError::new(500, "Token generation failed", Some(Box::new(json!({"error": msg}))))),
    ),
    JweError::InvalidKeyFormat(msg) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(WebError::new(500, "JWE key format error", Some(Box::new(json!({"error": msg}))))),
    ),
    _ => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(WebError::new(500, "Internal server error", Some(Box::new(json!({"error": e.to_string()}))))),
    ),
  })?;

  // 计算过期时间
  let now = Utc::now();
  let expires_at = now.timestamp() + jwe_config.token_ttl as i64;
  let issued_at = now.to_rfc3339();

  Ok(Json(GenerateTokenResponse {
    token,
    agent_id: agent_id.clone(),
    token_type: "Bearer".to_string(),
    expires_at,
    issued_at,
  }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::net::{IpAddr, Ipv4Addr, SocketAddr};

  #[test]
  fn test_is_localhost() {
    // IPv4 localhost
    let localhost_v4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
    assert!(is_localhost(&localhost_v4));

    let localhost_127 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    assert!(is_localhost(&localhost_127));

    // IPv6 localhost
    let localhost_v6 = SocketAddr::new(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), 8080);
    assert!(is_localhost(&localhost_v6));

    // Non-localhost
    let remote = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 8080);
    assert!(!is_localhost(&remote));
  }
}
