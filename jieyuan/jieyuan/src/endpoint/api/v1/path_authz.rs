use axum::{Json, extract::State, http::request::Parts};
use fusion_common::ctx::Ctx;
use fusion_core::application::Application;
use fusion_web::{WebError, WebResult, extract_ctx, ok_json};
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{
  CtxExt, PathLookupRequest,
  path_authz::{PathAuthzCtxPayload, PathBasedAuthzRequest, PathBasedAuthzResponse},
};

use crate::service::PathMappingSvc;

/// 基于路径的授权管理路由
pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new().routes(utoipa_axum::routes!(authorize_by_path))
}

/// 基于路径的权限检查
#[utoipa::path(
  post,
  path = "/authorize-by-path",
  request_body = PathBasedAuthzRequest,
  responses(
    (status = 200, description = "授权检查成功", body = PathBasedAuthzResponse),
    (status = 400, description = "请求参数错误"),
    (status = 401, description = "认证失败"),
    (status = 403, description = "权限不足"),
    (status = 404, description = "路径映射不存在")
  ),
  tag = "路径授权"
)]
async fn authorize_by_path(
  parts: Parts,
  State(_app): State<Application>,
  path_mapping_svc: PathMappingSvc,
  Json(req): Json<PathBasedAuthzRequest>,
) -> WebResult<PathBasedAuthzResponse> {
  // 从请求中提取用户上下文
  let ctx = extract_ctx(&parts, _app.fusion_setting().security())?;

  // 查找路径映射
  let lookup_req =
    PathLookupRequest { service: req.service.clone(), path: req.path.clone(), method: req.method.clone() };

  let lookup_response = path_mapping_svc
    .lookup_path(&lookup_req)
    .await
    .map_err(|e| WebError::bad_gateway(e.to_string()))?
    .ok_or_else(|| WebError::forbidden("no path mapping found"))?;

  // 记录审计日志
  log_permission_check(&req, &ctx, "allow").await?;

  // 构建用户上下文载荷
  let ctx_payload = PathAuthzCtxPayload {
    tenant_id: ctx.tenant_id(),
    sub: ctx.user_id(),
    principal_roles: ctx.roles().into_iter().map(|s| s.to_string()).collect(),
    is_platform_admin: ctx.is_platform_admin(),
    token_seq: ctx.token_seq(),
    method: req.method.clone(),
    path: req.path.clone(),
    request_ip: req.request_ip.unwrap_or_default(),
    req_time: ctx.req_epoch_secs().to_string(),
  };

  // 构建响应
  ok_json!(PathBasedAuthzResponse {
    decision: "allow".to_string(),
    ctx: Some(ctx_payload),
    matched_mapping: Some(jieyuan_core::model::path_authz::MatchedMapping {
      action: lookup_response.action,
      resource_tpl: lookup_response.resource_tpl,
      extracted_params: lookup_response.path_params,
    }),
  })
}

/// 记录权限检查日志（简化版本）
async fn log_permission_check(req: &PathBasedAuthzRequest, auth_ctx: &Ctx, decision: &str) -> Result<(), WebError> {
  log::info!(
    "Permission check: service={}, path={}, method={}, user_id={}, decision={}",
    req.service,
    req.path,
    req.method,
    auth_ctx.user_id(),
    decision
  );

  Ok(())
}
