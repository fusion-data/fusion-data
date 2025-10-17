use std::collections::HashMap;

use crate::model::auth_ctx::AuthContext;
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

/// 统一的资源模板渲染函数
/// 支持内置占位符和可选的自定义占位符，支持双层格式
///
/// # 参数
/// - `tpl`: 资源模板字符串
/// - `ac`: 授权上下文
/// - `extras`: 可选的自定义占位符映射，如果不需要自定义占位符则传入 `None`
///
/// # 示例
/// ```rust
/// // 仅使用内置占位符
/// let resource = render_resource("jr:hetumind:workflow/{id}", &ac, None);
///
/// // 使用自定义占位符
/// let mut extras = HashMap::new();
/// extras.insert("id".to_string(), "123".to_string());
/// let resource = render_resource("jr:hetumind:workflow/{id}", &ac, Some(&extras));
/// ```
pub fn render_resource(tpl: &str, ac: &AuthContext, extras: Option<&HashMap<String, String>>) -> String {
  let mut result = tpl.to_string();

  // 检查模板是否已包含 tenant_id 占位符
  if result.contains("{tenant_id}") {
    // 完整格式：直接替换占位符
    result = result.replace("{tenant_id}", &ac.principal_tenant_id.to_string());
  } else {
    // 简化格式：自动注入 tenant_id
    if let Some(colon_pos) = result.find(':')
      && let Some(second_colon_pos) = result[colon_pos + 1..].find(':')
    {
      let insert_pos = colon_pos + 1 + second_colon_pos + 1;
      result.insert_str(insert_pos, &format!("{}:", ac.principal_tenant_id));
    }
  }

  // 替换其他内置占位符
  result = result.replace("{user_id}", &ac.principal_user_id.to_string());
  result = result.replace("{method}", &ac.method);
  result = result.replace("{path}", &ac.path);
  result = result.replace("{token_seq}", &ac.token_seq.to_string());

  // 处理角色拼接
  if result.contains("{principal_roles}") {
    let joined = ac.principal_roles.join(",");
    result = result.replace("{principal_roles}", &joined);
  }

  // 处理自定义占位符（可选）
  if let Some(extras) = extras {
    for (k, v) in extras.iter() {
      let ph = format!("{{{}}}", k);
      if result.contains(&ph) {
        result = result.replace(&ph, v);
      }
    }
  }

  result
}

#[cfg(feature = "with-web")]
impl axum::response::IntoResponse for AuthorizeResponse {
  fn into_response(self) -> axum::response::Response {
    let body = axum::Json(self);
    (axum::http::StatusCode::OK, body).into_response()
  }
}
