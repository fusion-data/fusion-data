use axum::{
  extract::{Path, State},
  http::StatusCode,
  Json,
  response::IntoResponse,
};
use fusion_common::ctx::Ctx;
use fusion_core::application::Application;
use fusion_web::WebError;
use serde_json::json;

use super::{auth_ctx::build_auth_context_with_timezone, PolicySvc};

#[derive(serde::Serialize)]
struct OkResp {
  ok: bool,
}

/// 示例端点：用户更新密码
pub async fn update_user_endpoint(
  State(policy_svc): State<PolicySvc>,
  State(app): State<Application>,
  Path(user_id): Path<i64>,
  ctx: Ctx,
) -> Result<Json<OkResp>, WebError> {
  // 构建授权上下文
  let ac = build_auth_context_with_timezone(&ctx, *app.fusion_setting().app().time_offset())
    .map_err(|e| WebError::bad_request(e.to_string()))?;

  // 示例动作与资源
  let action = "user:update_password";

  // 将端点参数解析后作为 extras 占位符注入
  let mut extras = std::collections::HashMap::new();
  extras.insert("user_id", user_id.to_string());
  let resource = super::auth_ctx::render_resource_ext("jr:user:{tenant_id}:{user_id}", &ac, &extras);

  // 调用授权
  policy_svc
    .authorize(&ac, action, &resource)
    .map_err(|e| WebError::unauthorized(e.to_string()))?;

  Ok(Json(OkResp { ok: true }))
}

/// 示例端点：角色附加策略
pub async fn attach_policy_to_role_endpoint(
  State(policy_svc): State<PolicySvc>,
  State(app): State<Application>,
  Path((role_id, policy_id)): Path<(i64, i64)>,
  ctx: Ctx,
) -> Result<Json<OkResp>, WebError> {
  let ac = build_auth_context_with_timezone(&ctx, *app.fusion_setting().app().time_offset())
    .map_err(|e| WebError::bad_request(e.to_string()))?;

  let action = "policy:attach";

  let mut extras = std::collections::HashMap::new();
  extras.insert("role_id", role_id.to_string());
  extras.insert("policy_id", policy_id.to_string());
  let resource = super::auth_ctx::render_resource_ext("jr:role:{tenant_id}:{role_id}", &ac, &extras);

  policy_svc
    .authorize(&ac, action, &resource)
    .map_err(|e| WebError::unauthorized(e.to_string()))?;

  // 实际的角色策略附加逻辑...

  Ok(Json(OkResp { ok: true }))
}

/// 示例路由注册器
pub fn register_routes(policy_svc: PolicySvc) -> axum::Router {
  use super::{route_with_meta, RouteSpec};

  // 使用宏绑定元数据的方式
  let router = route_with_meta!(
    axum::Router::new(),
    axum::routing::put,
    "/api/v1/users/:user_id/password",
    update_user_endpoint,
    "user:update_password",
    "jr:user:{tenant_id}:{user_id}"
  );

  // 使用路由规格的方式
  let specs = vec![
    RouteSpec {
      path: "/api/v1/roles/:role_id/policies/:policy_id/attach",
      method: "POST",
      action: "policy:attach",
      resource_tpl: "jr:role:{tenant_id}:{role_id}",
      handler: attach_policy_to_role_endpoint,
    },
  ];

  let router_with_specs = super::build_router(specs);

  router
    .merge(router_with_specs)
    .with_state(policy_svc)
}

/// 示例：在应用启动时配置路由
pub fn configure_app_routes(app: &Application) -> axum::Router {
  let policy_svc = PolicySvc::with_mm(app.component::<fusionsql::ModelManager>());

  let api_router = register_routes(policy_svc);

  axum::Router::new()
    .nest("/api/v1", api_router)
    .with_state(app.clone())
}

/// 示例策略文档
pub const EXAMPLE_POLICY_ALLOW_SELF_UPDATE: &str = r#"
{
  "version": "2025-01-01",
  "id": "pol-allow-self-change-password",
  "statement": [
    {
      "sid": "self_change_password",
      "effect": "allow",
      "action": ["user:update_password"],
      "resource": ["jr:user:{tenant_id}:{user_id}"],
      "condition": {
        "string_equals": {
          "jr:principal_user_id": "{user_id}",
          "jr:tenant_id": "{tenant_id}"
        }
      }
    }
  ]
}
"#;

pub const EXAMPLE_POLICY_DENY_PLATFORM_ADMIN: &str = r#"
{
  "version": "2025-01-01",
  "id": "pol-deny-platform-admin-self-modify",
  "statement": [
    {
      "sid": "deny_platform_admin_self_modify",
      "effect": "deny",
      "action": ["user:*"],
      "resource": ["jr:user:*:*"],
      "condition": {
        "bool": {
          "jr:is_platform_admin": true
        }
      }
    }
  ]
}
"#;

/// 示例：测试路由的集成
#[cfg(test)]
mod tests {
  use super::*;
  use axum::{
    body::Body,
    http::{Method, Request},
  };
  use tower::ServiceExt;

  #[tokio::test]
  async fn test_unauthorized_request() {
    // 这里需要设置测试环境
    // 由于需要 Ctx 和 PolicySvc 的模拟，这是一个简化示例

    let policy_svc = PolicySvc::with_mm(fusionsql::ModelManager::for_testing());
    let router = register_routes(policy_svc);

    let request = Request::builder()
      .method(Method::PUT)
      .uri("/api/v1/users/1001/password")
      .header("Content-Type", "application/json")
      .body(Body::empty())
      .unwrap();

    // 在实际测试中，需要设置认证中间件和模拟的 Ctx
    // 这里仅展示请求构建方式
  }
}