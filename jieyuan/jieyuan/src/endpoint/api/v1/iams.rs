use axum::{extract::Json, http::request::Parts};
use fusion_common::ctx::Ctx;
use fusion_core::application::Application;
use fusion_web::{WebError, WebResult};
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{AuthorizeRequest, AuthorizeResponse, Decision, render_resource};

use crate::access_control::{PolicySvc, ResourceMappingSvc};

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new().routes(utoipa_axum::routes!(authorize))
}

/// 远程授权 API 端点
#[utoipa::path(
  post,
  path = "/authorize",
  tag = "IAM",
  summary = "远程授权",
  description = "执行远程授权检查，包括令牌验证和策略评估。所有请求必须通过路径映射表获取 action 和 resource_tpl，确保统一性和唯一性。\n\n**必需参数：**\n- `service`: 服务名称\n- `path`: 请求路径\n- `method`: HTTP 方法\n\n**可选参数：**\n- `path_code`: 路径代码（替代 service/path/method 组合）\n- `extras`: 变量替换参数（用于路径、资源、条件模板变量替换）\n- `request_ip`: 客户端 IP\n\n**查找优先级：**\n1. 如果提供了 `path_code`，直接查找对应的映射\n2. 否则使用 `service + path + method` 组合查找\n\n**extras 参数说明：**\n- 用于替换路径、资源、条件中的模板变量\n- 支持 `project_id`, `project_member_id`, `created_by` 等业务参数\n- 与路径映射提取的参数合并，映射提取的参数优先级更高",
  request_body = AuthorizeRequest,
  responses(
    (status = 200, description = "授权通过", body = AuthorizeResponse),
    (status = 400, description = "请求参数错误（缺少必需参数）", body = WebError),
    (status = 401, description = "令牌无效", body = WebError),
    (status = 403, description = "策略拒绝", body = AuthorizeResponse),
    (status = 404, description = "路径映射不存在", body = WebError),
    (status = "default", response = WebError)
  ),
  security(
    ("bearer_auth" = [])
  )
)]
pub async fn authorize(
  parts: Parts,
  policy_svc: PolicySvc,
  path_mapping_svc: ResourceMappingSvc,
  Json(req): Json<AuthorizeRequest>,
) -> WebResult<AuthorizeResponse> {
  // 1) 从请求 extensions 中提取用户上下文
  let ctx: &Ctx = parts.extensions.get().ok_or_else(|| WebError::new_with_code(401, "invalid token"))?;

  // 2) 路径映射查找：使用 path_code
  let lookup_response = path_mapping_svc
    .lookup_by_code(&req.path_code)
    .await
    .map_err(|e| WebError::new_with_code(404, format!("path code lookup failed: {}", e)))?
    .ok_or_else(|| WebError::new_with_code(404, format!("no path mapping found for path_code={}", req.path_code)))?;

  // 3) 合并变量替换参数（extras 用于路径、资源、条件的模板变量替换）
  // 路径映射提取的参数优先级高于客户端提供的参数
  let mut extras = req.extras.unwrap_or_default();
  for (key, value) in lookup_response.mapping_params {
    extras.insert(key, value);
  }

  let final_action = lookup_response.action;
  let final_resource_tpl = lookup_response.resource_tpl;
  let final_extras = Some(extras);

  // 3) 渲染资源模板
  let resource = render_resource(&final_resource_tpl, ctx, final_extras.as_ref());

  // 4) 记录授权日志
  log::info!(
    "Authorization check: action={}, resource={}, user_id={}, tenant_id={}",
    final_action,
    resource,
    ctx.user_id(),
    ctx.tenant_id()
  );

  // 5) 执行授权检查
  match policy_svc.authorize_ext(ctx, &final_action, &resource).await {
    Ok(Decision::Allow) => {
      // 授权通过，构建成功响应
      let response = AuthorizeResponse::success(ctx.clone());
      Ok(axum::Json(response))
    }
    Ok(Decision::Deny) => {
      // 授权拒绝，构建拒绝详情（用于日志）
      let response = AuthorizeResponse::denied(ctx.clone());
      let detail = serde_json::to_value(response)?;

      // 返回 403 Forbidden 响应
      let error_response =
        WebError::new(403, format!("policy deny: {} not allowed on {}", final_action, resource), Some(detail));

      Err(error_response)
    }
    Err(e) => {
      // 业务逻辑错误，转换为 401 Unauthorized
      Err(WebError::new_with_code(401, format!("authorization failed: {}", e)))
    }
  }
}
