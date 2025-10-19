use std::collections::HashMap;

use fusion_common::ctx::Ctx;
use serde::{Deserialize, Serialize};

use crate::model::{CtxExt, DecisionEffect};

/// 远程授权请求体（简化版）- 路径映射模式
/// 所有权限检查通过路径映射表获取 action 和 resource_tpl
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct AuthorizeRequest {
  /// 路径代码 - 用于直接查找特定映射，替代 service/path/method 组合
  pub path_code: String,

  /// 变量替换参数（可选）- 用于替换路径、资源、条件中的模板变量
  /// 如：project_id, project_member_id, created_by 等
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extras: Option<HashMap<String, String>>,

  /// 客户端 IP（可选）
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
  pub ctx: Ctx,
}

/// 用于反序列化的 AuthorizeResponse 辅助结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct AuthorizeResponseData {
  /// 授权决策结果
  pub decision: DecisionEffect,
  /// 授权上下文信息（序列化后的JSON）
  pub ctx: serde_json::Value,
}

impl AuthorizeRequest {
  pub fn new(path_code: impl Into<String>) -> Self {
    Self { path_code: path_code.into(), extras: None, request_ip: None }
  }

  /// 设置变量替换参数
  pub fn with_extras(mut self, extras: HashMap<String, String>) -> Self {
    self.extras = Some(extras);
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
  pub fn success(ctx: Ctx) -> Self {
    Self { decision: DecisionEffect::Allow, ctx }
  }

  /// 创建拒绝的授权响应
  pub fn denied(ctx: Ctx) -> Self {
    Self { decision: DecisionEffect::Deny, ctx }
  }
}

/// 统一的资源模板渲染函数
/// 支持内置占位符和可选的自定义占位符，支持混合架构
///
/// # 参数
/// - `tpl`: 资源模板字符串（简化格式，不包含 tenant_id）
/// - `ac`: 授权上下文
/// - `extras`: 可选的自定义占位符映射，如果不需要自定义占位符则传入 `None`
/// - `target_tenant_id`: 可选的目标租户ID，用于平台管理员跨租户访问
/// ```
pub fn render_resource(
  tpl: &str,
  ac: &Ctx,
  extras: Option<&HashMap<String, String>>,
  target_tenant_id: Option<i64>,
) -> String {
  let mut result = tpl.to_string();

  // 根据用户类型决定是否注入 tenant_id
  if ac.is_platform_admin() {
    match ac.tenant_access_mode() {
      crate::model::policy::TenantAccessMode::All => {
        // 平台管理员可访问所有租户，不注入 tenant_id
      }
      crate::model::policy::TenantAccessMode::Specific => {
        // 特定租户访问模式，使用指定租户ID
        if let Some(tenant_id) = target_tenant_id.or(ac.get_tenant_id()) {
          if !result.contains("{tenant_id}") {
            if let Some(colon_pos) = result.find(':')
              && let Some(second_colon_pos) = result[colon_pos + 1..].find(':')
            {
              let insert_pos = colon_pos + 1 + second_colon_pos + 1;
              result.insert_str(insert_pos, &format!("{}:", tenant_id));
            }
          }
        }
      }
      crate::model::policy::TenantAccessMode::Current => {
        // 当前租户模式，使用当前租户ID
        if !result.contains("{tenant_id}") {
          if let Some(colon_pos) = result.find(':')
            && let Some(second_colon_pos) = result[colon_pos + 1..].find(':')
          {
            let insert_pos = colon_pos + 1 + second_colon_pos + 1;
            result.insert_str(insert_pos, &format!("{}:", ac.tenant_id()));
          }
        }
      }
    }
  } else {
    // 普通用户：自动注入当前租户ID
    if !result.contains("{tenant_id}") {
      if let Some(colon_pos) = result.find(':')
        && let Some(second_colon_pos) = result[colon_pos + 1..].find(':')
      {
        let insert_pos = colon_pos + 1 + second_colon_pos + 1;
        result.insert_str(insert_pos, &format!("{}:", ac.tenant_id()));
      }
    }
  }

  // 替换 tenant_id 占位符（如果存在）
  if result.contains("{tenant_id}") {
    let tenant_id = target_tenant_id.unwrap_or(ac.tenant_id());
    result = result.replace("{tenant_id}", &tenant_id.to_string());
  }

  // 替换其他内置占位符
  result = result.replace("{user_id}", &ac.user_id().to_string());
  result = result.replace("{method}", ac.request_method());
  result = result.replace("{path}", ac.request_path());
  result = result.replace("{token_seq}", &ac.token_seq().to_string());

  // 处理角色拼接
  if result.contains("{principal_roles}") {
    let joined = ac.roles().join(",");
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
