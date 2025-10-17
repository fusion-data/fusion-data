//! 工作流 API 示例 - 展示如何集成 jieyuan 远程授权
//!
//! 这个文件展示了如何在 hetumind-studio 中使用 jieyuan 的远程授权功能
//! 来保护 API 端点。

use axum::{
  extract::{Path, State},
  http::request::Parts,
  Json,
};
use fusion_core::application::Application;
use fusion_web::WebResult;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use crate::web::remote_authz_middleware::{route_with_authz, route_with_authz_and_extras};

/// 带权限控制的工作流 API 示例
pub fn routes(app: Application) -> axum::Router {
  let router = axum::Router::new();

  // 示例 1: 基础的权限控制路由
  let router = route_with_authz!(
    router,
    axum::routing::get,
    "/api/v1/workflows",
    list_workflows_handler,
    "hetumind:list",
    "jr:hetumind:workflow/*"
  );

  // 示例 2: 带参数的权限控制路由
  let router = route_with_authz_and_extras!(
    router,
    axum::routing::get,
    "/api/v1/workflows/:id",
    get_workflow_handler,
    "hetumind:read",
    "jr:hetumind:workflow/{id}",
    HashMap::new() // 这里会自动被中间件填充
  );

  // 示例 3: 创建工作流的权限控制
  let router = route_with_authz!(
    router,
    axum::routing::post,
    "/api/v1/workflows",
    create_workflow_handler,
    "hetumind:create",
    "jr:hetumind:workflow/*"
  );

  // 示例 4: 更新工作流的权限控制（需要特定的工作流ID）
  let router = axum::Router::new()
    .route(
      "/api/v1/workflows/:id",
      axum::routing::put(update_workflow_handler),
    )
    .route_layer(axum::middleware::from_fn_with_args(
      crate::web::remote_authz_middleware::inject_extras,
      HashMap::from([("id".to_string(), "".to_string())]), // 占位符，实际会被中间件替换
    ))
    .route_layer(axum::middleware::from_fn_with_args(
      crate::web::remote_authz_middleware::inject_route_meta,
      "hetumind:update",
      "jr:hetumind:workflow/{id}",
    ))
    .route_layer(axum::middleware::from_fn(
      crate::web::remote_authz_middleware::remote_authz_guard,
    ));

  // 添加应用状态
  router.with_state(app)
}

/// 获取用户上下文的辅助函数
fn get_user_context(parts: &Parts) -> Result<crate::web::remote_authz_middleware::CtxPayloadView, WebError> {
  parts.extensions.get::<crate::web::remote_authz_middleware::CtxPayloadView>()
    .cloned()
    .ok_or_else(|| WebError::unauthorized("missing user context"))
}

/// 列出工作流处理器
pub async fn list_workflows_handler(
  parts: Parts,
  State(_app): State<Application>,
) -> WebResult<Json<Value>> {
  // 从中间件注入的用户上下文中获取用户信息
  let ctx = get_user_context(&parts)?;

  // 这里可以基于 ctx.tenant_id 和 ctx.user_id 进行业务逻辑处理
  let response = json!({
    "workflows": [],
    "user_id": ctx.user_id(),
    "tenant_id": ctx.tenant_id(),
    "roles": ctx.principal_roles,
    "message": "Successfully listed workflows (permission: hetumind:list)"
  });

  Ok(Json(response))
}

/// 获取单个工作流处理器
pub async fn get_workflow_handler(
  parts: Parts,
  State(_app): State<Application>,
  Path(workflow_id): Path<Uuid>,
) -> WebResult<Json<Value>> {
  // 获取用户上下文
  let ctx = get_user_context(&parts)?;

  // 检查权限 - 确保用户有权限访问这个特定的工作流
  let response = json!({
    "id": workflow_id.to_string(),
    "name": "Example Workflow",
    "user_id": ctx.user_id(),
    "tenant_id": ctx.tenant_id(),
    "roles": ctx.principal_roles,
    "message": format!("Successfully accessed workflow {} (permission: hetumind:read)", workflow_id)
  });

  Ok(Json(response))
}

/// 创建工作流处理器
pub async fn create_workflow_handler(
  parts: Parts,
  State(_app): State<Application>,
  Json(payload): Json<Value>,
) -> WebResult<Json<Value>> {
  // 获取用户上下文
  let ctx = get_user_context(&parts)?;

  // 检查用户是否有创建权限（hetumind:create）
  if !ctx.has_role("editor") && !ctx.has_role("admin") {
    return Err(WebError::forbidden("Insufficient permissions to create workflow"));
  }

  let response = json!({
    "id": Uuid::new_v4().to_string(),
    "name": payload.get("name").unwrap_or(&json!("Untitled Workflow")),
    "created_by": ctx.user_id(),
    "tenant_id": ctx.tenant_id(),
    "message": "Successfully created workflow (permission: hetumind:create)"
  });

  Ok(Json(response))
}

/// 更新工作流处理器
pub async fn update_workflow_handler(
  parts: Parts,
  State(_app): State<Application>,
  Path(workflow_id): Path<Uuid>,
  Json(payload): Json<Value>,
) -> WebResult<Json<Value>> {
  // 获取用户上下文
  let ctx = get_user_context(&parts)?;

  // 检查用户是否有更新权限（hetumind:update）
  // 注意：这里的权限检查已经在远程授权中间件中完成了
  // 中间件确保了用户只能访问有权限的工作流

  let response = json!({
    "id": workflow_id.to_string(),
    "updated_by": ctx.user_id(),
    "tenant_id": ctx.tenant_id(),
    "changes": payload,
    "message": format!("Successfully updated workflow {} (permission: hetumind:update)", workflow_id)
  });

  Ok(Json(response))
}

/// 复杂权限检查示例
pub async fn advanced_workflow_handler(
  parts: Parts,
  State(_app): State<Application>,
  Path((workflow_id, action)): Path<(Uuid, String)>,
) -> WebResult<Json<Value>> {
  // 获取用户上下文
  let ctx = get_user_context(&parts)?;

  // 根据不同的操作进行权限检查
  let required_permission = match action.as_str() {
    "execute" => "hetumind:execute",
    "duplicate" => "hetumind:duplicate",
    "share" => "hetumind:share",
    "activate" => "hetumind:activate",
    "deactivate" => "hetumind:deactivate",
    _ => return Err(WebError::bad_request("Invalid action")),
  };

  // 检查用户是否有相应权限
  let has_permission = ctx.has_role("admin") ||
    (ctx.has_role("editor") && matches!(action.as_str(), "execute" | "duplicate"));

  if !has_permission {
    return Err(WebError::forbidden(format!(
      "Insufficient permissions for action: {}", required_permission
    )));
  }

  let response = json!({
    "id": workflow_id.to_string(),
    "action": action,
    "performed_by": ctx.user_id(),
    "tenant_id": ctx.tenant_id(),
    "permission": required_permission,
    "message": format!("Successfully performed {} on workflow {}", action, workflow_id)
  });

  Ok(Json(response))
}