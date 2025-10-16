use chrono::{DateTime, FixedOffset, Utc};
use fusion_core::{DataError, Result};
use fusion_common::ctx::Ctx;
use serde::{Deserialize, Serialize};

/// 授权上下文（强类型视图），用于策略求值
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthContext {
  pub principal_user_id: i64,
  pub principal_tenant_id: i64,
  pub principal_roles: Vec<i64>,
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

  // principal_roles 约定为字符串数组，转换为 i64 列表
  let roles: Vec<i64> = ctx
    .payload()
    .get_strings("principal_roles")
    .unwrap_or_default()
    .into_iter()
    .filter_map(|s| s.parse::<i64>().ok())
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
    build_auth_context(ctx).map_err(|e| e)
  }
}