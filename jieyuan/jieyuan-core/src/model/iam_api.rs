use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 远程授权请求体（snake_case）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct AuthorizeRequest {
  /// 行为名，格式 {service}:{verb}
  pub action: String,
  /// 资源模板，支持内置占位符与路由参数占位符
  pub resource_tpl: String,
  /// 路由参数或业务参数的显式占位符值
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extras: Option<HashMap<String, String>>,
  /// HTTP 方法小写
  #[serde(skip_serializing_if = "Option::is_none")]
  pub method: Option<String>,
  /// 当前请求路径
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
  /// 客户端 IP
  #[serde(skip_serializing_if = "Option::is_none")]
  pub request_ip: Option<String>,
}

/// 远程授权响应体（成功）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct AuthorizeResponse {
  /// 授权决策结果
  pub decision: DecisionEffect,
  /// 授权上下文信息
  pub ctx: CtxPayload,
}

/// 授权上下文信息（CtxPayload）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct CtxPayload {
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

// 重新导出 policy 模块中的 DecisionEffect，作为远程 API 使用的类型
pub use super::policy::DecisionEffect;

/// 授权拒绝响应详情
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct AuthorizeDenyDetail {
  /// 决策结果（拒绝）
  pub decision: DecisionEffect,
  /// 授权上下文信息（用于日志）
  pub ctx: CtxPayload,
}

impl AuthorizeRequest {
  /// 创建新的授权请求
  pub fn new(action: impl Into<String>, resource_tpl: impl Into<String>) -> Self {
    Self {
      action: action.into(),
      resource_tpl: resource_tpl.into(),
      extras: None,
      method: None,
      path: None,
      request_ip: None,
    }
  }

  /// 设置 extras 参数
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

  /// 设置客户端 IP
  pub fn with_request_ip(mut self, request_ip: impl Into<String>) -> Self {
    self.request_ip = Some(request_ip.into());
    self
  }
}

impl AuthorizeResponse {
  /// 创建成功的授权响应
  pub fn success(ctx: CtxPayload) -> Self {
    Self { decision: DecisionEffect::Allow, ctx }
  }

  /// 创建拒绝的授权响应
  pub fn denied(ctx: CtxPayload) -> Self {
    Self { decision: DecisionEffect::Deny, ctx }
  }
}

impl CtxPayload {
  /// 创建新的上下文载荷
  pub fn new(
    tenant_id: i64,
    sub: i64,
    principal_roles: Vec<String>,
    is_platform_admin: bool,
    token_seq: i32,
    method: String,
    path: String,
    request_ip: String,
    req_time: String,
  ) -> Self {
    Self { tenant_id, sub, principal_roles, is_platform_admin, token_seq, method, path, request_ip, req_time }
  }
}

impl AuthorizeDenyDetail {
  /// 创建拒绝详情
  pub fn new(ctx: CtxPayload) -> Self {
    Self { decision: DecisionEffect::Deny, ctx }
  }
}

impl CtxPayload {
  /// 从 fusion_common::ctx::Ctx 创建 CtxPayload
  #[cfg(feature = "with-web")]
  pub fn from_ctx(ctx: &fusion_common::ctx::Ctx) -> Self {
    let roles: Vec<String> = ctx
      .payload()
      .get_strings("principal_roles")
      .unwrap_or_default()
      .into_iter()
      .map(|s| s.to_string())
      .collect();

    Self {
      tenant_id: ctx.tenant_id(),
      sub: ctx.uid(),
      principal_roles: roles,
      is_platform_admin: ctx.payload().get_bool("is_platform_admin").unwrap_or(false),
      token_seq: ctx.payload().get_i32("token_seq").unwrap_or(0),
      method: ctx.payload().get_str("method").unwrap_or("").to_string(),
      path: ctx.payload().get_str("path").unwrap_or("").to_string(),
      request_ip: ctx.payload().get_str("request_ip").unwrap_or("").to_string(),
      req_time: format!("{:?}", ctx.req_time()), // 暂时使用 Debug 格式
    }
  }
}

#[cfg(feature = "with-web")]
impl axum::response::IntoResponse for AuthorizeResponse {
  fn into_response(self) -> axum::response::Response {
    let body = axum::Json(self);
    (axum::http::StatusCode::OK, body).into_response()
  }
}
