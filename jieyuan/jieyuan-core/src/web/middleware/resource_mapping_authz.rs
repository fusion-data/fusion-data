use std::net::SocketAddr;

use axum::{
  extract::{ConnectInfo, Request, State},
  http::header,
  middleware::Next,
  response::Response,
};
use fusion_core::application::Application;
use fusion_web::WebError;

use crate::{model::AuthorizeRequest, web::JieyuanClient};

/// 简化的资源映射权限中间件
/// 自动从请求中提取必要信息并调用 jieyuan 权限检查
pub async fn resource_mapping_authz_middleware(
  State(app): State<Application>,
  mut req: Request<axum::body::Body>,
  next: Next,
) -> Result<Response, WebError> {
  // 1. 提取 Authorization 头
  let auth_header = req
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .ok_or_else(|| WebError::unauthorized("missing Authorization header"))?;

  // 2. 提取请求信息
  let path = req.uri().path().to_string();
  let method = req.method().to_string().to_lowercase();
  let request_ip = extract_client_ip(&req);

  // 3. 调用 jieyuan 资源映射权限检查
  let jieyuan_client = app.component::<JieyuanClient>();
  let authz_response = jieyuan_client
    .authorize(auth_header, AuthorizeRequest::new("hetumind", &path, &method).with_request_ip(request_ip))
    .await?;

  // 4. 注入用户上下文
  if let Some(ctx) = authz_response.ctx {
    req.extensions_mut().insert(ctx);
  }

  // 5. 继续处理请求
  Ok(next.run(req).await)
}

/// 基于映射代码的权限中间件
/// 直接使用映射代码进行权限检查，适用于已知映射代码的场景
pub async fn resource_mapping_authz_by_code_middleware(
  State(app): State<Application>,
  mut req: Request<axum::body::Body>,
  next: Next,
) -> Result<Response, WebError> {
  // 1. 提取 Authorization 头
  let auth_header = req
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .ok_or_else(|| WebError::unauthorized("missing Authorization header"))?;

  // 2. 从请求中提取映射代码（可以从 header 或 path parameter 中获取）
  let mapping_code = req
    .headers()
    .get("x-mapping-code")
    .and_then(|v| v.to_str().ok())
    .or_else(|| {
      // 也可以从路径参数中提取，例如 /api/v1/resource/{mapping_code}/action
      let path = req.uri().path();
      if let Some(code_start) = path.rfind("/resource/") {
        let remaining = &path[code_start + 10..]; // +10 跳过 "/resource/"
        if let Some(code_end) = remaining.find('/') {
          remaining[..code_end].to_string().into()
        } else if !remaining.is_empty() {
          remaining.to_string().into()
        } else {
          None
        }
      } else {
        None
      }
    })
    .ok_or_else(|| WebError::bad_request("missing mapping code"))?;

  let request_ip = extract_client_ip(&req);

  // 3. 调用 jieyuan 资源映射权限检查
  let jieyuan_client = app.component::<JieyuanClient>();
  let authz_response = jieyuan_client
    .authorize(
      auth_header,
      AuthorizeRequest::new("hetumind").with_mapping_code(mapping_code).with_request_ip(request_ip),
    )
    .await?;

  // 4. 注入用户上下文
  if let Some(ctx) = authz_response.ctx {
    req.extensions_mut().insert(ctx);
  }

  // 5. 继续处理请求
  Ok(next.run(req).await)
}

/// 提取客户端 IP
fn extract_client_ip(req: &Request<axum::body::Body>) -> String {
  req
    .headers()
    .get("x-forwarded-for")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.split(',').next())
    .map(|s| s.to_string())
    .or_else(|| req.headers().get("x-real-ip").and_then(|v| v.to_str().map(|s| s.to_string()).ok()))
    .or_else(|| req.extensions().get::<ConnectInfo<SocketAddr>>().map(|addr| addr.ip().to_string()))
    .unwrap_or_default()
}

/// 资源映射授权上下文
#[derive(Debug, Clone)]
pub struct ResourceMappingAuthzContext {
  pub user_id: i64,
  pub tenant_id: i64,
  pub roles: Vec<String>,
  pub decision: String,
  pub matched_mapping: Option<crate::model::resource_mapping_authz::MatchedMapping>,
}
