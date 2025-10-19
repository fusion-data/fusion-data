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

/// 简化的路径权限中间件
/// 自动从请求中提取必要信息并调用 jieyuan 权限检查
pub async fn path_authz_middleware(
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
  let path_code = "TODO";
  let request_ip = extract_client_ip(&req);

  // 3. 调用 jieyuan 路径权限检查
  let jieyuan_client = app.component::<JieyuanClient>();
  let authz_response = jieyuan_client
    .authorize(auth_header, AuthorizeRequest::new(path_code).with_request_ip(request_ip))
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
