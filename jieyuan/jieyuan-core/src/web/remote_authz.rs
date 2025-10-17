//! 远程授权客户端集成
//!
//! 这个模块提供了在其他项目（如 hetumind-studio、hetuflow-server）
//! 中集成和使用 jieyuan 的远程授权 API 的客户端功能。

use axum::{
  extract::{Request, State},
  http::{StatusCode, header},
  middleware::Next,
  response::Response,
};
use fusion_core::application::Application;
use fusion_web::WebError;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::model::iam_api::IamCtxPayload;
use crate::model::{AuthorizeRequest, AuthorizeResponse};

/// 远程授权客户端配置
#[derive(Clone)]
pub struct RemoteAuthzConfig {
  /// jieyuan 服务的基础 URL
  pub jieyuan_base_url: String,
  /// 请求超时时间（毫秒）
  pub timeout_ms: u64,
}

impl Default for RemoteAuthzConfig {
  fn default() -> Self {
    Self { jieyuan_base_url: "http://localhost:50010".to_string(), timeout_ms: 5000 }
  }
}

/// 远程授权中间件（简化版本）
///
/// 这个中间件会：
/// 1. 读取 Authorization 头
/// 2. 构造授权请求体（snake_case），action 和 resource_tpl 将在远程端通过路径映射自动解析
/// 3. 调用远程授权 API 并处理 200/403/401
/// 4. 在 200 allow 时，将 ctx 注入到请求扩展供后续 handler 使用
pub async fn remote_authz_guard(
  State(_app): State<Application>,
  State(config): State<RemoteAuthzConfig>,
  mut req: Request<axum::body::Body>,
  next: Next,
) -> Result<Response, WebError> {
  // 1) 读取 Authorization 头
  let auth_header = req
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .ok_or_else(|| WebError::unauthorized("missing Authorization header"))?;

  // 2) 使用简化的路径授权（新方案不再需要 RouteMeta）
  // 动作和资源模板将通过路径映射在远程端自动解析

  // 读取 extras（路由参数或业务参数）
  let extras = req.extensions().get::<HashMap<String, String>>().cloned().unwrap_or_default();

  // 3) 组装请求体（snake_case）
  let method = req.method().to_string().to_lowercase();
  let path = req.uri().path().to_string();
  let request_ip = req
    .headers()
    .get("x-forwarded-for")
    .and_then(|v| v.to_str().ok())
    .or_else(|| req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()))
    .unwrap_or("")
    .to_string();

  // 新的简化方案：action 和 resource_tpl 将在远程端通过路径映射自动解析
  let body = json!({
    "extras": extras,
    "method": method,
    "path": path,
    "request_ip": request_ip,
  });

  // 4) 远程调用 Jieyuan 授权 API
  let url = format!("{}/api/v1/iam/authorize", config.jieyuan_base_url);
  let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_millis(config.timeout_ms))
    .build()
    .map_err(|e| WebError::bad_gateway(format!("failed to create HTTP client: {}", e)))?;

  let resp = client
    .post(&url)
    .header(header::AUTHORIZATION, auth_header)
    .header(header::CONTENT_TYPE, "application/json")
    .json(&body)
    .send()
    .await
    .map_err(|e| WebError::bad_gateway(format!("authorize request failed: {}", e)))?;

  // 5) 响应处理（200/403/401）
  let status = resp.status();
  let bytes = resp.bytes().await.map_err(|e| WebError::bad_gateway(e.to_string()))?;

  match status {
    StatusCode::OK => {
      // 约定：{ decision: "allow", ctx: {...} }
      let json: Value = serde_json::from_slice(&bytes).map_err(|e| WebError::bad_gateway(e.to_string()))?;

      // 决策防御性检查（decision 复用 DecisionEffect，snake_case）
      let decision = json.get("decision").and_then(|v| v.as_str()).unwrap_or("deny");

      if decision != "allow" {
        return Err(WebError::forbidden("policy deny (unexpected decision)"));
      }

      // 将 ctx 注入请求上下文
      if let Some(ctx) = json.get("ctx") {
        req.extensions_mut().insert(ctx.clone());
      }

      Ok(next.run(req).await)
    }
    StatusCode::FORBIDDEN => {
      // 约定：返回 JSON 错误响应，转换为 WebError
      let error_text = String::from_utf8_lossy(&bytes);
      Err(WebError::forbidden(error_text))
    }
    StatusCode::UNAUTHORIZED => {
      let error_text = String::from_utf8_lossy(&bytes);
      Err(WebError::unauthorized(error_text))
    }
    _ => Err(WebError::bad_gateway(format!("unexpected status: {}", status))),
  }
}

/// 从 JSON 值创建 IamCtxPayload
pub fn ctx_payload_from_json(value: &Value) -> Result<IamCtxPayload, serde_json::Error> {
  serde_json::from_value(value.clone())
}

/// 授权请求参数
#[derive(Debug, Clone)]
pub struct AuthorizeParams {
  /// 认证令牌
  pub token: String,
  /// 动作
  pub action: String,
  /// 资源模板
  pub resource_tpl: String,
  /// 额外参数
  pub extras: Option<HashMap<String, String>>,
  /// HTTP 方法
  pub method: Option<String>,
  /// 请求路径
  pub path: Option<String>,
  /// 请求 IP
  pub request_ip: Option<String>,
}

impl AuthorizeParams {
  /// 创建新的授权参数
  pub fn new(token: impl Into<String>, action: impl Into<String>, resource_tpl: impl Into<String>) -> Self {
    Self {
      token: token.into(),
      action: action.into(),
      resource_tpl: resource_tpl.into(),
      extras: None,
      method: None,
      path: None,
      request_ip: None,
    }
  }

  /// 设置额外参数
  pub fn with_extras(mut self, extras: HashMap<String, String>) -> Self {
    self.extras = Some(extras);
    self
  }

  /// 设置 HTTP 方法
  pub fn with_method(mut self, method: impl Into<String>) -> Self {
    self.method = Some(method.into());
    self
  }

  /// 设置请求路径
  pub fn with_path(mut self, path: impl Into<String>) -> Self {
    self.path = Some(path.into());
    self
  }

  /// 设置请求 IP
  pub fn with_request_ip(mut self, request_ip: impl Into<String>) -> Self {
    self.request_ip = Some(request_ip.into());
    self
  }
}

/// 远程授权客户端
/// 提供便捷的方法来调用远程授权 API
pub struct RemoteAuthzClient {
  config: RemoteAuthzConfig,
  client: reqwest::Client,
}

impl RemoteAuthzClient {
  /// 创建新的远程授权客户端
  pub fn new(config: RemoteAuthzConfig) -> Result<Self, WebError> {
    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_millis(config.timeout_ms))
      .build()
      .map_err(|e| WebError::bad_gateway(format!("failed to create HTTP client: {}", e)))?;

    Ok(Self { config, client })
  }

  /// 执行远程授权检查
  pub async fn authorize(&self, params: AuthorizeParams) -> Result<AuthorizeResponse, WebError> {
    let url = format!("{}/api/v1/iam/authorize", self.config.jieyuan_base_url);

    let request = AuthorizeRequest::new(&params.action, &params.resource_tpl)
      .with_extras(params.extras.unwrap_or_default())
      .with_method(params.method.unwrap_or_default())
      .with_path(params.path.unwrap_or_default())
      .with_request_ip(params.request_ip.unwrap_or_default());

    let resp = self
      .client
      .post(&url)
      .bearer_auth(&params.token)
      .json(&request)
      .send()
      .await
      .map_err(|e| WebError::bad_gateway(format!("authorize request failed: {}", e)))?;

    let status = resp.status();
    if status.is_success() {
      let response: AuthorizeResponse =
        resp.json().await.map_err(|e| WebError::bad_gateway(format!("failed to parse response: {}", e)))?;
      Ok(response)
    } else {
      let error_text = resp.text().await.unwrap_or_else(|_| "unknown error".to_string());

      match status.as_u16() {
        401 => Err(WebError::unauthorized(format!("unauthorized: {}", error_text))),
        403 => Err(WebError::forbidden(format!("forbidden: {}", error_text))),
        _ => Err(WebError::bad_gateway(format!("remote authz error ({}): {}", status, error_text))),
      }
    }
  }
}
