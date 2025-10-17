//! 权限集成示例和配置指南
//!
//! 这个模块展示了如何在 hetumind-studio 中集成和使用 jieyuan 的远程授权系统。

use axum::{
  extract::{Path, State},
  http::request::Parts,
  Json,
  middleware,
};
use fusion_core::application::Application;
use fusion_web::WebResult;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::web::remote_authz_middleware::*;

/// 展示如何构建带权限控制的路由器
pub fn build_authz_routes(app: Application) -> axum::Router {
  let mut router = axum::Router::new();

  // 使用宏简化路由注册 - 基础示例
  router = route_with_authz!(
    router,
    axum::routing::get,
    "/api/v1/projects",
    list_projects_handler,
    "hetumind:list",
    "jr:hetumind:project/*"
  );

  // 使用宏简化路由注册 - 带参数示例
  router = route_with_authz!(
    router,
    axum::routing::get,
    "/api/v1/projects/:project_id/workflows",
    list_project_workflows_handler,
    "hetumind:list",
    "jr:hetumind:project/{project_id}/workflow/*"
  );

  // 使用宏简化路由注册 - 创建操作
  router = route_with_authz!(
    router,
    axum::routing::post,
    "/api/v1/workflows",
    create_workflow_handler,
    "hetumind:create",
    "jr:hetumind:workflow/*"
  );

  // 使用宏简化路由注册 - 特定资源操作
  router = route_with_authz!(
    router,
    axum::routing::put,
    "/api/v1/workflows/:workflow_id",
    update_workflow_handler,
    "hetumind:update",
    "jr:hetumind:workflow/{workflow_id}"
  );

  // 手动构建更复杂的中间件栈 - 高级示例
  router = router
    .route(
      "/api/v1/workflows/:workflow_id/execute",
      axum::routing::post(execute_workflow_handler),
    )
    // 步骤1: 注入路由参数作为 extras
    .route_layer(middleware::from_fn_with_args(
      inject_extras,
      HashMap::from([
        ("workflow_id".to_string(), "".to_string()), // 占位符
        ("action".to_string(), "execute".to_string()),
      ]),
    ))
    // 步骤2: 注入路由元数据
    .route_layer(middleware::from_fn_with_args(
      inject_route_meta,
      "hetumind:execute",
      "jr:hetumind:workflow/{workflow_id}",
    ))
    // 步骤3: 执行远程授权检查
    .route_layer(middleware::from_fn(remote_authz_guard));

  router.with_state(app)
}

/// 获取当前用户上下文
fn get_current_user_context(parts: &Parts) -> Result<CtxPayloadView, WebError> {
  parts.extensions.get::<CtxPayloadView>()
    .cloned()
    .ok_or_else(|| WebError::unauthorized("用户上下文缺失"))
}

/// 项目列表处理器
pub async fn list_projects_handler(
  parts: Parts,
  State(_app): State<Application>,
) -> WebResult<Json<Value>> {
  let ctx = get_current_user_context(&parts)?;

  let response = json!({
    "projects": [
      {
        "id": "proj-001",
        "name": "AI工作流项目",
        "description": "使用AI驱动的自动化工作流"
      },
      {
        "id": "proj-002",
        "name": "数据处理项目",
        "description": "ETL数据处理管道"
      }
    ],
    "user": {
      "id": ctx.user_id(),
      "tenant_id": ctx.tenant_id(),
      "roles": ctx.principal_roles,
      "is_platform_admin": ctx.is_platform_admin()
    },
    "permission": "hetumind:list",
    "resource": "jr:hetumind:project/*"
  });

  Ok(Json(response))
}

/// 项目工作流列表处理器
pub async fn list_project_workflows_handler(
  parts: Parts,
  State(_app): State<Application>,
  Path(project_id): Path<String>,
) -> WebResult<Json<Value>> {
  let ctx = get_current_user_context(&parts)?;

  // 检查用户是否可以访问该项目
  // 这里可以添加额外的业务逻辑检查

  let response = json!({
    "project_id": project_id,
    "workflows": [
      {
        "id": "wf-001",
        "name": "数据清洗工作流",
        "status": "active"
      },
      {
        "id": "wf-002",
        "name": "报告生成工作流",
        "status": "draft"
      }
    ],
    "user": {
      "id": ctx.user_id(),
      "tenant_id": ctx.tenant_id(),
      "roles": ctx.principal_roles
    },
    "permission": "hetumind:list",
    "resource": format!("jr:hetumind:project/{}/workflow/*", project_id)
  });

  Ok(Json(response))
}

/// 创建工作流处理器
pub async fn create_workflow_handler(
  parts: Parts,
  State(_app): State<Application>,
  Json(payload): Json<Value>,
) -> WebResult<Json<Value>> {
  let ctx = get_current_user_context(&parts)?;

  // 业务逻辑：检查用户权限和业务规则
  let can_create = ctx.has_role("editor") || ctx.has_role("admin") || ctx.is_platform_admin();

  if !can_create {
    return Err(WebError::forbidden("您没有创建工作流的权限"));
  }

  let workflow_name = payload.get("name")
    .and_then(|v| v.as_str())
    .unwrap_or("未命名工作流");

  let response = json!({
    "id": format!("wf-{}", uuid::Uuid::new_v4().simple()),
    "name": workflow_name,
    "status": "draft",
    "created_by": {
      "id": ctx.user_id(),
      "tenant_id": ctx.tenant_id()
    },
    "created_at": chrono::Utc::now().to_rfc3339(),
    "permission": "hetumind:create",
    "resource": "jr:hetumind:workflow/*"
  });

  Ok(Json(response))
}

/// 更新工作流处理器
pub async fn update_workflow_handler(
  parts: Parts,
  State(_app): State<Application>,
  Path(workflow_id): Path<String>,
  Json(payload): Json<Value>,
) -> WebResult<Json<Value>> {
  let ctx = get_current_user_context(&parts)?;

  // 注意：权限检查已经在中间件中完成
  // 这里可以添加额外的业务逻辑验证

  let response = json!({
    "id": workflow_id,
    "updated_fields": payload,
    "updated_by": {
      "id": ctx.user_id(),
      "tenant_id": ctx.tenant_id()
    },
    "updated_at": chrono::Utc::now().to_rfc3339(),
    "permission": "hetumind:update",
    "resource": format!("jr:hetumind:workflow/{}", workflow_id)
  });

  Ok(Json(response))
}

/// 执行工作流处理器 - 高级示例
pub async fn execute_workflow_handler(
  parts: Parts,
  State(_app): State<Application>,
  Path(workflow_id): Path<String>,
  Json(payload): Json<Value>,
) -> WebResult<Json<Value>> {
  let ctx = get_current_user_context(&parts)?;

  // 检查额外权限
  let can_execute = ctx.has_role("admin") ||
    (ctx.has_role("editor") && payload.get("force_execute").is_none());

  if !can_execute {
    return Err(WebError::forbidden("您没有执行工作流的权限"));
  }

  let response = json!({
    "execution_id": format!("exec-{}", uuid::Uuid::new_v4().simple()),
    "workflow_id": workflow_id,
    "status": "running",
    "executed_by": {
      "id": ctx.user_id(),
      "tenant_id": ctx.tenant_id(),
      "roles": ctx.principal_roles
    },
    "started_at": chrono::Utc::now().to_rfc3339(),
    "permission": "hetumind:execute",
    "resource": format!("jr:hetumind:workflow/{}", workflow_id),
    "config": payload
  });

  Ok(Json(response))
}

/// 权限配置示例
pub const AUTHZ_CONFIG_EXAMPLE: &str = r#"
# hetumind-studio 权限集成配置示例

# 1. 环境变量配置
export JIEYUAN_BASE_URL="http://localhost:50010"
export JIEYUAN_TIMEOUT_MS="5000"

# 2. 应用配置 (TOML)
[hetermind.authz]
jieyuan_base_url = "http://localhost:50010"
timeout_ms = 5000

# 3. 示例权限策略 (在 jieyuan 中配置)
{
  "version": "2025-01-01",
  "id": "hetumind-workflow-policy",
  "statement": [
    {
      "sid": "viewer_access",
      "effect": "allow",
      "action": ["hetumind:read", "hetumind:list"],
      "resource": ["jr:hetumind:{tenant_id}:workflow/*"],
      "condition": {
        "string_equals": {
          "jr:principal_roles": ["viewer", "editor", "admin"]
        }
      }
    },
    {
      "sid": "editor_access",
      "effect": "allow",
      "action": ["hetumind:create", "hetumind:update", "hetumind:execute"],
      "resource": ["jr:hetumind:{tenant_id}:workflow/*"],
      "condition": {
        "string_equals": {
          "jr:principal_roles": ["editor", "admin"]
        }
      }
    },
    {
      "sid": "admin_access",
      "effect": "allow",
      "action": ["hetumind:*"],
      "resource": ["jr:hetumind:{tenant_id}:*"],
      "condition": {
        "string_equals": {
          "jr:principal_roles": ["admin"]
        }
      }
    }
  ]
}

# 4. 资源模板示例
# 基础资源模板
"jr:hetumind:workflow/*" -> 所有工作流
"jr:hetumind:project/{project_id}" -> 特定项目
"jr:hetumind:workflow/{workflow_id}" -> 特定工作流
"jr:hetumind:project/{project_id}/workflow/*" -> 项目内所有工作流

# 5. 权限操作示例
"hetumind:list" - 列表查看
"hetumind:read" - 详情查看
"hetumind:create" - 创建资源
"hetumind:update" - 更新资源
"hetumind:delete" - 删除资源
"hetumind:execute" - 执行工作流
"hetumind:duplicate" - 复制工作流
"hetumind:share" - 分享资源
"#;