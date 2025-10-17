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

/// 远程授权中间件
///
/// 这个中间件会：
/// 1. 读取 Authorization 头和 RouteMeta
/// 2. 构造授权请求体（snake_case）
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

  // 2) 读取路由元数据（动作与资源模板）
  let meta = req
    .extensions()
    .get::<super::route_meta::RouteMeta>()
    .ok_or_else(|| WebError::bad_request("missing route meta"))?;

  // 3) 读取 extras（路由参数或业务参数）
  let extras = req.extensions().get::<HashMap<String, String>>().cloned().unwrap_or_default();

  // 4) 组装请求体（snake_case）
  let method = req.method().to_string().to_lowercase();
  let path = req.uri().path().to_string();
  let request_ip = req
    .headers()
    .get("x-forwarded-for")
    .and_then(|v| v.to_str().ok())
    .or_else(|| req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()))
    .unwrap_or("")
    .to_string();

  let body = json!({
    "action": meta.action,
    "resource_tpl": meta.resource_tpl,
    "extras": extras,
    "method": method,
    "path": path,
    "request_ip": request_ip,
  });

  // 5) 远程调用 Jieyuan 授权 API
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

  // 6) 响应处理（200/403/401）
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

/// 客户端上下文载荷视图
/// 用于在客户端项目中映射远程返回的 ctx
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CtxPayloadView {
  /// 租户 ID
  pub tenant_id: i64,
  /// 用户 ID（sub）
  pub sub: i64,
  /// 用户角色编码列表
  pub principal_roles: Vec<String>,
  /// 是否平台管理员
  pub is_platform_admin: bool,
  /// 令牌序列号
  pub token_seq: i32,
  /// HTTP 方法
  pub method: String,
  /// 请求路径
  pub path: String,
  /// 客户端 IP
  pub request_ip: String,
  /// 请求时间（RFC3339 + FixedOffset）
  pub req_time: String,
}

impl CtxPayloadView {
  /// 从 JSON 值创建视图
  pub fn from_json(value: &Value) -> Result<Self, serde_json::Error> {
    serde_json::from_value(value.clone())
  }

  /// 获取用户 ID
  pub fn user_id(&self) -> i64 {
    self.sub
  }

  /// 获取租户 ID
  pub fn tenant_id(&self) -> i64 {
    self.tenant_id
  }

  /// 检查是否具有指定角色
  pub fn has_role(&self, role: &str) -> bool {
    self.principal_roles.contains(&role.to_string())
  }

  /// 检查是否是平台管理员
  pub fn is_platform_admin(&self) -> bool {
    self.is_platform_admin
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
  pub async fn authorize(
    &self,
    token: &str,
    action: &str,
    resource_tpl: &str,
    extras: Option<HashMap<String, String>>,
    method: Option<&str>,
    path: Option<&str>,
    request_ip: Option<&str>,
  ) -> Result<AuthorizeResponse, WebError> {
    let url = format!("{}/api/v1/iam/authorize", self.config.jieyuan_base_url);

    let request = AuthorizeRequest::new(action, resource_tpl)
      .with_extras(extras.unwrap_or_default())
      .with_method(method.unwrap_or(""))
      .with_path(path.unwrap_or(""))
      .with_request_ip(request_ip.unwrap_or(""));

    let resp = self
      .client
      .post(&url)
      .bearer_auth(token)
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
