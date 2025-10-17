//! 远程授权中间件
//!
//! 这个模块集成了 jieyuan 的远程授权功能，为 hetumind-studio 提供权限控制。

use axum::{
  extract::{Request, State},
  http::{header, StatusCode},
  middleware::Next,
  response::Response,
};
use fusion_core::application::Application;
use fusion_web::WebError;
use serde_json::Value;
use std::collections::HashMap;

use jieyuan_core::web::remote_authz::{RemoteAuthzConfig, CtxPayloadView};

/// 远程授权配置
pub fn get_remote_authz_config(_app: &Application) -> RemoteAuthzConfig {
  use fusion_common::env::{get_env, get_env_parse};

  // 默认配置
  let base_url = get_env("JIEYUAN_BASE_URL")
    .unwrap_or_else(|_| "http://localhost:50010".to_string());

  let timeout_ms: u64 = get_env_parse("JIEYUAN_TIMEOUT_MS")
    .unwrap_or(5000);

  RemoteAuthzConfig {
    jieyuan_base_url: base_url,
    timeout_ms,
  }
}

/// 远程授权中间件
///
/// 使用 jieyuan-core 中的 RouteMeta 和远程授权客户端
pub async fn remote_authz_guard(
  State(app): State<Application>,
  mut req: Request<axum::body::Body>,
  next: Next,
) -> Result<Response, WebError> {
  // 获取远程授权配置
  let config = get_remote_authz_config(&app);

  // 1) 读取 Authorization 头
  let auth_header = req
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .ok_or_else(|| WebError::unauthorized("missing Authorization header"))?;

  // 2) 读取路由元数据（动作与资源模板）
  let meta = req.extensions().get::<jieyuan_core::web::route_meta::RouteMeta>()
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

  let body = serde_json::json!({
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

      // 决策防御性检查
      let decision = json.get("decision").and_then(|v| v.as_str()).unwrap_or("deny");
      if decision != "allow" {
        return Err(WebError::forbidden("policy deny (unexpected decision)"));
      }

      // 将 ctx 注入请求上下文
      if let Some(ctx) = json.get("ctx") {
        // 转换为 CtxPayloadView 并注入
        if let Ok(ctx_view) = CtxPayloadView::from_json(ctx) {
          req.extensions_mut().insert(ctx_view);
        }
      }

      Ok(next.run(req).await)
    }
    StatusCode::FORBIDDEN => {
      // 约定：返回 JSON 错误响应
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

/// 路由元数据注入中间件
/// 使用 jieyuan-core 中的函数
pub async fn inject_route_meta(
  action: &'static str,
  resource_tpl: &'static str,
  mut req: Request<axum::body::Body>,
  next: Next,
) -> Response {
  use jieyuan_core::web::route_meta::RouteMeta;
  req.extensions_mut().insert(RouteMeta { action, resource_tpl });
  next.run(req).await
}

/// 扩展参数（extras）注入中间件
pub async fn inject_extras(
  extras: HashMap<String, String>,
  mut req: Request<axum::body::Body>,
  next: Next,
) -> Response {
  req.extensions_mut().insert(extras);
  next.run(req).await
}

/// 路由注册宏 - 简化带权限控制的路由注册
#[macro_export]
macro_rules! route_with_authz {
  ($router:expr, $method:path, $path:expr, $handler:path, $action:expr, $resource_tpl:expr) => {{
    use axum::{middleware, routing::MethodRouter};
    use std::collections::HashMap;

    $router
      .route($path, $method($handler))
      .route_layer(middleware::from_fn_with_args(
        crate::web::remote_authz_middleware::inject_route_meta,
        $action,
        $resource_tpl,
      ))
      .route_layer(middleware::from_fn(
        crate::web::remote_authz_middleware::remote_authz_guard,
      ))
  }};
}

/// 带额外参数的路由注册宏
#[macro_export]
macro_rules! route_with_authz_and_extras {
  ($router:expr, $method:path, $path:expr, $handler:path, $action:expr, $resource_tpl:expr, $extras:expr) => {{
    use axum::{middleware, routing::MethodRouter};
    use std::collections::HashMap;

    $router
      .route($path, $method($handler))
      .route_layer(middleware::from_fn_with_args(
        crate::web::remote_authz_middleware::inject_extras,
        $extras,
      ))
      .route_layer(middleware::from_fn_with_args(
        crate::web::remote_authz_middleware::inject_route_meta,
        $action,
        $resource_tpl,
      ))
      .route_layer(middleware::from_fn(
        crate::web::remote_authz_middleware::remote_authz_guard,
      ))
  }};
}