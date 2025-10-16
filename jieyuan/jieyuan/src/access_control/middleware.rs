use axum::{
  extract::{Request, State},
  http::Response,
  middleware::Next,
};
use fusion_common::ctx::Ctx;
use fusion_core::application::Application;
use fusion_web::WebError;

use crate::access_control::PolicySvc;
use crate::access_control::auth_ctx::{AuthContext, build_auth_context_with_timezone};

/// 路由元数据，用于绑定动作和资源模板
#[derive(Clone)]
pub struct RouteMeta {
  pub action: &'static str,
  pub resource_tpl: &'static str,
}

/// 函数级注释：最小授权中间件，将业务层 DataError 映射为 WebError
pub async fn authz_guard(
  State(policy_svc): State<PolicySvc>,
  State(app): State<Application>,
  ctx: Ctx,
  req: Request<axum::body::Body>,
  next: Next,
) -> Result<Response<axum::body::Body>, WebError> {
  let ac = build_auth_context_with_timezone(&ctx, *app.fusion_setting().app().time_offset())
    .map_err(|e| WebError::new_with_msg(e.to_string()))?;

  // 获取路由元数据
  let meta = req
    .extensions()
    .get::<RouteMeta>()
    .ok_or_else(|| WebError::new_with_msg("missing route meta".to_string()))?;

  // 渲染资源模板
  let resource = render_resource(meta.resource_tpl, &ac);

  // 执行授权检查
  policy_svc
    .authorize(&ac, meta.action, &resource)
    .await
    .map_err(|e| WebError::new_with_code(401, e.to_string()))?;

  Ok(next.run(req).await)
}

/// 函数级注释：简单的资源模板渲染（仅支持内置占位符）
fn render_resource(tpl: &str, ac: &AuthContext) -> String {
  tpl
    .replace("{tenant_id}", &ac.principal_tenant_id.to_string())
    .replace("{user_id}", &ac.principal_user_id.to_string())
    .replace("{method}", &ac.method)
    .replace("{path}", &ac.path)
    .replace("{token_seq}", &ac.token_seq.to_string())
}

/// 函数级注释：路由元数据注入中间件
pub async fn inject_route_meta(
  action: &'static str,
  resource_tpl: &'static str,
  mut req: Request<axum::body::Body>,
  next: Next,
) -> axum::response::Response {
  req.extensions_mut().insert(RouteMeta { action, resource_tpl });
  next.run(req).await
}
