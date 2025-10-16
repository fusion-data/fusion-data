use chrono::{DateTime, FixedOffset, Utc};
use fusion_common::ctx::Ctx;
use fusion_core::{DataError, Result};
use serde::{Deserialize, Serialize};

/// 授权上下文（强类型视图），用于策略求值
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthContext {
  pub principal_user_id: i64,
  pub principal_tenant_id: i64,
  pub principal_roles: Vec<String>,
  pub is_platform_admin: bool,
  pub token_seq: i32,
  pub request_ip: String,
  pub now: DateTime<FixedOffset>,
  pub method: String,
  pub path: String,
}

/// 函数级注释：将 Ctx 投影为授权求值视图 AuthContext
pub fn build_auth_context(ctx: &Ctx) -> Result<AuthContext> {
  let tz = FixedOffset::east_opt(0).ok_or_else(|| DataError::bad_request("invalid timezone"))?;
  let now = DateTime::<Utc>::from(*ctx.req_time()).with_timezone(&tz);

  // principal_roles 约定为字符串数组
  let roles: Vec<String> = ctx
    .payload()
    .get_strings("principal_roles")
    .unwrap_or_default()
    .into_iter()
    .map(|s| s.to_string())
    .collect();

  Ok(AuthContext {
    principal_user_id: ctx.uid(),
    principal_tenant_id: ctx.tenant_id(),
    principal_roles: roles,
    is_platform_admin: ctx.payload().get_bool("is_platform_admin").unwrap_or(false),
    token_seq: ctx.payload().get_i32("token_seq").unwrap_or(0),
    request_ip: ctx.payload().get_str("request_ip").unwrap_or("").to_string(),
    now,
    method: ctx.payload().get_str("method").unwrap_or("").to_string(),
    path: ctx.payload().get_str("path").unwrap_or("").to_string(),
  })
}

/// 函数级注释：将 Ctx 投影为授权求值视图 AuthContext，支持指定时区
pub fn build_auth_context_with_timezone(ctx: &Ctx, time_offset: FixedOffset) -> Result<AuthContext> {
  let now = DateTime::<Utc>::from(*ctx.req_time()).with_timezone(&time_offset);

  // principal_roles 约定为字符串数组
  let roles: Vec<String> = ctx
    .payload()
    .get_strings("principal_roles")
    .unwrap_or_default()
    .into_iter()
    .map(|s| s.to_string())
    .collect();

  Ok(AuthContext {
    principal_user_id: ctx.uid(),
    principal_tenant_id: ctx.tenant_id(),
    principal_roles: roles,
    is_platform_admin: ctx.payload().get_bool("is_platform_admin").unwrap_or(false),
    token_seq: ctx.payload().get_i32("token_seq").unwrap_or(0),
    request_ip: ctx.payload().get_str("request_ip").unwrap_or("").to_string(),
    now,
    method: ctx.payload().get_str("method").unwrap_or("").to_string(),
    path: ctx.payload().get_str("path").unwrap_or("").to_string(),
  })
}

/// 函数级注释：提供 TryFrom<&Ctx> 便捷转换接口
impl TryFrom<&Ctx> for AuthContext {
  type Error = DataError;
  fn try_from(ctx: &Ctx) -> std::result::Result<Self, Self::Error> {
    build_auth_context(ctx)
  }
}

/// 函数级注释：提供从 Ctx 与时区构建 AuthContext 的便捷方法
impl AuthContext {
  pub fn try_from_ctx(ctx: &Ctx, time_offset: FixedOffset) -> Result<Self> {
    build_auth_context_with_timezone(ctx, time_offset)
  }
}

/// 函数级注释：扩展资源模板渲染，支持内置与自定义占位符
pub fn render_resource_ext(tpl: &str, ac: &AuthContext, extras: &std::collections::HashMap<&str, String>) -> String {
  let mut s = tpl.to_string();
  // 内置占位符
  s = s
    .replace("{tenant_id}", &ac.principal_tenant_id.to_string())
    .replace("{user_id}", &ac.principal_user_id.to_string())
    .replace("{method}", &ac.method)
    .replace("{path}", &ac.path)
    .replace("{token_seq}", &ac.token_seq.to_string());

  // 角色（拼接为逗号分隔）
  if s.contains("{principal_roles}") {
    let joined = ac.principal_roles.join(",");
    s = s.replace("{principal_roles}", &joined);
  }

  // 其它自定义占位符（如 role_id/policy_id/resource_id 等）
  for (k, v) in extras.iter() {
    let ph = format!("{{{}}}", k);
    if s.contains(&ph) {
      s = s.replace(&ph, v);
    }
  }

  s
}
