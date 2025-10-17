use axum::{extract::Json, http::request::Parts};
use fusion_common::ctx::Ctx;
use fusion_core::application::Application;
use fusion_web::{WebError, WebResult};
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{AuthorizeRequest, AuthorizeResponse, Decision, render_resource};

use crate::access_control::PolicySvc;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new().routes(utoipa_axum::routes!(authorize))
}

/// 远程授权 API 端点
#[utoipa::path(
  post,
  path = "/iam/authorize",
  tag = "IAM",
  summary = "远程授权",
  description = "执行远程授权检查，包括令牌验证和策略评估",
  request_body = AuthorizeRequest,
  responses(
    (status = 200, description = "授权通过", body = AuthorizeResponse),
    (status = 401, description = "令牌无效", body = WebError),
    (status = 403, description = "策略拒绝", body = AuthorizeResponse),
    (status = "default", response = WebError)
  ),
  security(
    ("bearer_auth" = [])
  )
)]
pub async fn authorize(
  parts: Parts,
  policy_svc: PolicySvc,
  Json(req): Json<AuthorizeRequest>,
) -> WebResult<AuthorizeResponse> {
  // 1) 从请求 extensions 中提取用户上下文
  let ctx: &Ctx = parts.extensions.get().ok_or_else(|| WebError::new_with_code(401, "invalid token"))?;

  // 2) 渲染资源模板
  let resource = render_resource(&req.resource_tpl, &ctx, req.extras.as_ref());

  // 3) 执行授权检查
  match policy_svc.authorize_ext(&ctx, &req.action, &resource).await {
    Ok(Decision::Allow) => {
      // 授权通过，构建成功响应
      let response = AuthorizeResponse::success(ctx.clone());
      Ok(axum::Json(response))
    }
    Ok(Decision::Deny) => {
      // 授权拒绝，构建拒绝详情（用于日志）
      let _detail = jieyuan_core::model::AuthorizeDenyDetail::new(ctx.clone());

      // 返回 403 Forbidden 响应
      let error_response = WebError::new(403, format!("policy deny: {} not allowed on {}", req.action, resource), None);

      Err(error_response)
    }
    Err(e) => {
      // 业务逻辑错误，转换为 401 Unauthorized
      Err(WebError::new_with_code(401, format!("authorization failed: {}", e)))
    }
  }
}
