use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
  extract::{ConnectInfo, Request, State},
  http::header,
  middleware::Next,
  response::Response,
};
use fusion_core::application::Application;
use fusion_web::WebError;

use crate::{
  model::{AuthorizeRequest, TenantAccessValidator, TenantFilter},
  web::JieyuanClient,
};

/// 混合架构认证中间件
/// 支持普通用户和平台管理员的差异化权限处理
pub async fn mixed_authz_middleware(
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
  let path_code = extract_path_code(&req)?;
  let request_ip = extract_client_ip(&req);
  let extras = extract_request_extras(&req);

  // 3. 调用 jieyuan 权限检查
  let jieyuan_client = app.component::<JieyuanClient>();
  let authz_request = AuthorizeRequest::new(path_code).with_request_ip(request_ip).with_extras(extras);

  let authz_response = jieyuan_client.authorize(auth_header, authz_request).await?;

  // 4. 验证用户身份和权限
  let ctx = authz_response.ctx.ok_or_else(|| WebError::unauthorized("invalid authorization context"))?;

  // 5. 为平台管理员设置特殊处理
  let enhanced_ctx = enhance_context_for_platform_admin(ctx.clone(), &req).await?;

  // 6. 注入增强的上下文和租户过滤器
  req.extensions_mut().insert(enhanced_ctx.clone());
  req.extensions_mut().insert(TenantFilter::new(enhanced_ctx.clone()));
  req.extensions_mut().insert(TenantAccessValidator::new(enhanced_ctx.clone()));

  // 7. 继续处理请求
  Ok(next.run(req).await)
}

/// 平台管理员增强中间件
/// 为平台管理员提供额外的权限和能力
pub async fn platform_admin_enhancement_middleware(
  req: Request<axum::body::Body>,
  next: Next,
) -> Result<Response, WebError> {
  // 获取用户上下文
  let ctx = req
    .extensions()
    .get::<fusion_common::ctx::Ctx>()
    .cloned()
    .ok_or_else(|| WebError::unauthorized("missing user context"))?;

  // 只有平台管理员需要增强处理
  if ctx.is_platform_admin() {
    // 检查是否需要租户切换
    if let Some(target_tenant_id) = extract_target_tenant_id(&req) {
      let validator = TenantAccessValidator::new(ctx.clone());
      validator.validate_tenant_access(target_tenant_id)?;

      // 创建临时上下文用于跨租户访问
      let temp_ctx = create_cross_tenant_context(ctx, target_tenant_id)?;
      req.extensions_mut().insert(temp_ctx);
    }
  }

  Ok(next.run(req).await)
}

/// 增强用户上下文为平台管理员
async fn enhance_context_for_platform_admin(
  mut ctx: fusion_common::ctx::Ctx,
  req: &Request<axum::body::Body>,
) -> Result<fusion_common::ctx::Ctx, WebError> {
  if ctx.is_platform_admin() {
    // 从数据库或配置中获取平台管理员的租户访问权限
    // 这里简化处理，实际应该查询用户配置

    // 检查是否有特殊的租户访问配置
    if let Some(access_mode) = extract_tenant_access_mode(req) {
      ctx.payload_mut().set_string("tenant_access_mode", &access_mode);
    }

    // 检查是否有管理租户列表
    if let Some(managed_tenants) = extract_managed_tenant_list(req) {
      ctx.payload_mut().set_strings("managed_tenant_ids", managed_tenants);
    }

    // 设置平台管理员特权标志
    ctx.payload_mut().set_bool("platform_privileges_enabled", true);
  }

  Ok(ctx)
}

/// 从请求中提取路径代码
fn extract_path_code(req: &Request<axum::body::Body>) -> Result<String, WebError> {
  // 简化实现：从路径或头部获取路径代码
  // 实际应该从路由配置或路径映射表中获取
  let path = req.uri().path();
  let method = req.method().as_str();

  // 简单的路径到代码映射
  let path_code = match (method, path) {
    ("GET", p) if p.starts_with("/api/v1/workflows/") => "workflow.get",
    ("POST", "/api/v1/workflows") => "workflow.create",
    ("PUT", p) if p.starts_with("/api/v1/workflows/") => "workflow.update",
    ("DELETE", p) if p.starts_with("/api/v1/workflows/") => "workflow.delete",
    ("GET", p) if p.starts_with("/api/v1/credentials/") => "credential.get",
    ("POST", "/api/v1/credentials") => "credential.create",
    ("PUT", p) if p.starts_with("/api/v1/credentials/") => "credential.update",
    ("DELETE", p) if p.starts_with("/api/v1/credentials/") => "credential.delete",
    ("GET", p) if p.starts_with("/api/v1/executions/") => "execution.get",
    ("POST", "/api/v1/executions") => "execution.create",
    // 管理员专用路径
    ("GET", "/api/v1/admin/tenants") => "admin.tenant.list",
    ("POST", "/api/v1/admin/tenants") => "admin.tenant.create",
    ("PUT", p) if p.starts_with("/api/v1/admin/tenants/") => "admin.tenant.update",
    ("DELETE", p) if p.starts_with("/api/v1/admin/tenants/") => "admin.tenant.delete",
    ("GET", "/api/v1/admin/users") => "admin.user.list",
    ("POST", "/api/v1/admin/users") => "admin.user.create",
    ("PUT", p) if p.starts_with("/api/v1/admin/users/") => "admin.user.update",
    _ => return Err(WebError::unauthorized("unauthorized path")),
  };

  Ok(path_code.to_string())
}

/// 从请求中提取额外参数
fn extract_request_extras(req: &Request<axum::body::Body>) -> std::collections::HashMap<String, String> {
  let mut extras = std::collections::HashMap::new();

  // 从路径参数中提取ID
  let path = req.uri().path();
  if let Some(id) = extract_id_from_path(path) {
    extras.insert("id".to_string(), id);
  }

  // 从查询参数中提取
  if let Some(query) = req.uri().query() {
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
      extras.insert(key.to_string(), value.to_string());
    }
  }

  // 从头部中提取相关信息
  if let Some(tenant_header) = req.headers().get("x-target-tenant-id") {
    if let Ok(tenant_id) = tenant_header.to_str() {
      extras.insert("target_tenant_id".to_string(), tenant_id.to_string());
    }
  }

  extras
}

/// 从路径中提取ID
fn extract_id_from_path(path: &str) -> Option<String> {
  let segments: Vec<&str> = path.split('/').collect();
  for segment in segments {
    if segment.chars().all(|c| c.is_ascii_digit() || c == '-') {
      return Some(segment.to_string());
    }
  }
  None
}

/// 提取目标租户ID
fn extract_target_tenant_id(req: &Request<axum::body::Body>) -> Option<i64> {
  // 从头部获取
  if let Some(tenant_header) = req.headers().get("x-target-tenant-id") {
    if let Ok(tenant_str) = tenant_header.to_str() {
      return tenant_str.parse().ok();
    }
  }

  // 从查询参数获取
  if let Some(query) = req.uri().query() {
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
      if key == "tenant_id" {
        return value.parse().ok();
      }
    }
  }

  None
}

/// 提取租户访问模式
fn extract_tenant_access_mode(req: &Request<axum::body::Body>) -> Option<String> {
  // 从头部获取
  if let Some(mode_header) = req.headers().get("x-tenant-access-mode") {
    mode_header.to_str().ok().map(|s| s.to_string())
  } else {
    None
  }
}

/// 提取管理租户列表
fn extract_managed_tenant_list(req: &Request<axum::body::Body>) -> Option<Vec<String>> {
  // 从头部获取
  if let Some(tenants_header) = req.headers().get("x-managed-tenant-ids") {
    if let Ok(tenants_str) = tenants_header.to_str() {
      return Some(tenants_str.split(',').map(|s| s.trim().to_string()).collect());
    }
  }
  None
}

/// 创建跨租户访问的临时上下文
fn create_cross_tenant_context(
  mut ctx: fusion_common::ctx::Ctx,
  target_tenant_id: i64,
) -> Result<fusion_common::ctx::Ctx, WebError> {
  // 验证平台管理员权限
  if !ctx.is_platform_admin() {
    return Err(WebError::unauthorized("only platform administrators can access cross-tenant resources"));
  }

  // 创建临时上下文用于跨租户访问
  ctx.payload_mut().set_i64("original_tenant_id", ctx.tenant_id());
  ctx.payload_mut().set_i64("target_tenant_id", target_tenant_id);
  ctx.payload_mut().set_bool("cross_tenant_access", true);

  Ok(ctx)
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

/// 权限检查辅助函数
pub mod authz_helpers {
  use super::*;
  use axum::extract::Request;
  use fusion_common::ctx::Ctx;

  /// 检查请求是否有平台管理员权限
  pub fn requires_platform_admin(req: &Request<axum::body::Body>) -> bool {
    let path = req.uri().path();
    path.starts_with("/api/v1/admin/") || path.starts_with("/api/v1/platform/")
  }

  /// 检查请求是否需要租户验证
  pub fn requires_tenant_validation(req: &Request<axum::body::Body>) -> bool {
    let path = req.uri().path();
    // 管理员路径可能需要特殊处理
    !path.starts_with("/api/v1/admin/tenants") && !path.starts_with("/api/v1/admin/users")
  }

  /// 获取请求的租户上下文类型
  pub fn get_tenant_context_type(ctx: &Ctx) -> &'static str {
    if ctx.is_platform_admin() {
      match ctx.tenant_access_mode() {
        crate::model::policy::TenantAccessMode::All => "platform_admin_all",
        crate::model::policy::TenantAccessMode::Current => "platform_admin_current",
        crate::model::policy::TenantAccessMode::Specific => "platform_admin_specific",
      }
    } else {
      "normal_user"
    }
  }
}
