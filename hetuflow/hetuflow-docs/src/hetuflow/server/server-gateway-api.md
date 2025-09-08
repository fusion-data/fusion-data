# Gateway API 设计文档

## API 接口设计概览

网关的 API 接口基于 `ServerApplication` 架构，提供 RESTful API 来管理 Agent、Job、Task、TaskInstance，监控连接状态和系统健康状况。所有 API 都位于 `/api/v1` 路径下，并按功能模块分层组织。

## 1. 当前实现状态

### 已实现的核心组件

- **应用层 (Application Layer)**: `ServerApplication` - 主应用容器，管理所有服务组件
- **网关层 (Gateway Layer)**:
  - `GatewaySvc` - WebSocket 网关服务
  - `ConnectionManager` - 连接管理器
  - `MessageHandler` - 消息路由器
- **服务层 (Service Layer)**:
  - `SchedulerSvc` - 调度服务
  - `AgentManager` - Agent 管理器
  - `TaskGenerationSvc` - 任务生成服务（预生成/事件驱动），与 TaskPoller+SchedulerSvc 协同
- **数据访问层 (Data Access Layer)**:
  - 各种 BMC (Basic Model Controller) 实现
  - 基于 `modelsql` 框架的类型安全数据库操作
- **模型层 (Model Layer)**:
  - 完整的实体模型定义 (Agent, Job, Task, TaskInstance, Server, Schedule)
  - 支持创建、更新、过滤的数据结构

### 待实现的组件

- **API 端点层**: 目前 `src/endpoint` 目录结构存在但实现为空
- **HTTP 路由**: 需要实现基于 Axum 的 RESTful API
- **认证授权**: API 安全机制
- **API 文档**: OpenAPI/Swagger 文档生成

## 2. API 端点设计

代码将在 [src/endpoint](../../../fusion/hetuflow-server/src/endpoint/) 模块下实现

### Agent 管理 API (`/api/v1/agents`)

- `POST /api/v1/agents/query` - 查询 Agent 列表（支持分页和过滤）
- `POST /api/v1/agents/create` - 创建新 Agent
- `GET /api/v1/agents/{id}` - 获取特定 Agent 详情
- `POST /api/v1/agents/{id}/update` - 更新 Agent 信息
- `DELETE /api/v1/agents/{id}` - 删除 Agent
- `POST /api/v1/agents/{id}/logs` - 获取 Agent 日志
- `POST /api/v1/agents/{id}/status` - 获取 Agent 实时状态
- `POST /api/v1/agents/{id}/commands` - 向 Agent 发送指令

### Job 管理 API (`/api/v1/jobs`)

- `POST /api/v1/jobs/query` - 查询 Job 列表（支持分页和过滤）
- `POST /api/v1/jobs/create` - 创建新 Job
- `GET /api/v1/jobs/{id}` - 获取特定 Job 详情
- `POST /api/v1/jobs/{id}/update` - 更新 Job 信息
- `DELETE /api/v1/jobs/{id}` - 删除 Job
- `POST /api/v1/jobs/{id}/enable` - 启用 Job
- `POST /api/v1/jobs/{id}/disable` - 禁用 Job

### Task 管理 API (`/api/v1/tasks`)

- `POST /api/v1/tasks/query` - 查询 Task 列表（支持分页和过滤）
- `POST /api/v1/tasks/create` - 创建新 Task
- `GET /api/v1/tasks/{id}` - 获取特定 Task 详情
- `POST /api/v1/tasks/{id}/update` - 更新 Task 信息
- `DELETE /api/v1/tasks/{id}` - 删除 Task
- `POST /api/v1/tasks/{id}/retry` - 重试 Task
- `POST /api/v1/tasks/{id}/cancel` - 取消 Task

### TaskInstance 管理 API (`/api/v1/task-instances`)

- `POST /api/v1/task-instances/query` - 查询 TaskInstance 列表（支持分页和过滤）
- `POST /api/v1/task-instances/create` - 创建新 TaskInstance
- `GET /api/v1/task-instances/{id}` - 获取特定 TaskInstance 详情
- `POST /api/v1/task-instances/{id}/update` - 更新 TaskInstance 信息
- `DELETE /api/v1/task-instances/{id}` - 删除 TaskInstance
- `POST /api/v1/task-instances/{id}/start` - 启动 TaskInstance
- `POST /api/v1/task-instances/{id}/complete` - 完成 TaskInstance
- `POST /api/v1/task-instances/{id}/cancel` - 取消 TaskInstance

### 连接管理 API (`/api/v1/connections`)

- `POST /api/v1/connections/query` - 获取当前连接列表
- `POST /api/v1/connections/stats` - 获取连接统计信息
- `DELETE /api/v1/connections/{agent_id}` - 强制断开特定 Agent 连接
- `POST /api/v1/connections/broadcast` - 向所有在线 Agent 广播消息

### 系统监控 API (`/api/v1/system`)

- `GET /api/v1/health` - 健康检查
- `POST /api/v1/metrics` - 系统指标
- `POST /api/v1/info` - 系统信息

## 3. 核心实现代码

### 3.1 统一响应类型和错误处理

基于现有架构的数据结构：

```rust
// 分页和过滤支持
use modelsql::{
  filter::page::Page,
  page::PageResult,
  ModelManager,
  SqlError,
};

// Web 响应类型
use fusion_web::WebResult;
use fusion_core::DataError;

// 应用核心
use crate::ServerApplication;
```

### 3.2 数据模型和 BMC 导入

所有数据模型和 BMC 已在代码库中实现，API 实现时直接导入使用：

```rust
// 数据模型 - 已在 src/model/ 中实现
use hetuflow_core::models::{
  SchedAgent, AgentForCreate, AgentForUpdate, AgentForQuery,
  SchedJob, JobForCreate, JobForUpdate, JobFilter, JobForQuery,
  SchedTask, TaskForCreate, TaskForUpdate, TaskFilter, TaskForQuery,
  SchedTaskInstance, TaskInstanceForCreate, TaskInstanceForUpdate, TaskInstanceFilter, TaskInstanceForQuery,
  SchedServer, SchedSchedule, ServerForQuery,
};

// BMC 数据访问层 - 已在 src/infra/bmc/ 中实现
use crate::infra::bmc::{
  AgentBmc, JobBmc, TaskBmc, TaskInstanceBmc, ServerBmc, ScheduleBmc,
};

// ServerApplication - 已在 src/application.rs 中实现
use crate::ServerApplication;
```

### 2.2 Agent 管理 API 实现

```rust
// src/endpoint/agents/mod.rs
pub mod handlers;

use axum::{routing::*, Router};
use std::sync::Arc;
use crate::ServerApplication;

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/query", post(handlers::query_agents))
    .route("/create", post(handlers::create_agent))
    .route("/:id", get(handlers::get_agent).delete(handlers::delete_agent))
    .route("/:id/update", post(handlers::update_agent))
    .route("/:id/logs", post(handlers::get_agent_logs))
    .route("/:id/status", post(handlers::get_agent_status))
    .route("/:id/commands", post(handlers::send_agent_command))
    .route("/online", get(handlers::list_online_agents))
    .route("/offline", get(handlers::list_offline_agents))
    .route("/:id/heartbeat", post(handlers::update_agent_heartbeat))
}

// src/endpoint/agents/handlers.rs
use axum::{
  extract::{Path, State},
  Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
  ServerApplication,
  endpoint::types::*,
  model::agent::*,
  infra::bmc::AgentBmc,
};
use hetuflow_core::{
  protocol::{AgentCommand, WebSocketMessage},
  types::AgentStatus,
};

/// 查询 Agent 列表
///
/// 根据提供的过滤条件和分页参数查询 Agent 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: 包含过滤条件和分页参数的查询请求
///
/// # 返回
/// - `PageResult<SchedAgent>`: 分页的 Agent 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn query_agents(
  State(app): State<ServerApplication>,
  Json(request): Json<AgentForQuery>,
) -> WebResult<PageResult<SchedAgent>> {
  todo!()
}

/// 创建新 Agent
///
/// 根据提供的参数创建新的 Agent
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: AgentForCreate 创建请求参数
///
/// # 返回
/// - `SchedAgent`: 创建的 Agent 实体
///
/// # 错误
/// - Agent ID 已存在时返回冲突错误
/// - 数据库操作失败时返回相应错误
pub async fn create_agent(
  State(app): State<ServerApplication>,
  Json(request): Json<AgentForCreate>,
) -> WebResult<SchedAgent> {
  todo!()
}

/// 获取特定 Agent 详情
///
/// 根据 Agent ID 获取 Agent 的详细信息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `agent_id`: Agent 的唯一标识符
///
/// # 返回
/// - `SchedAgent`: Agent 实体详情
///
/// # 错误
/// - Agent 不存在时返回 404 错误
/// - 数据库查询失败时返回相应错误
pub async fn get_agent(
  State(app): State<ServerApplication>,
  Path(agent_id): Path<Uuid>,
) -> WebResult<SchedAgent> {
  todo!()
}

/// 更新 Agent 信息
///
/// 根据提供的参数更新指定 Agent 的信息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `agent_id`: Agent 的唯一标识符
/// - `request`: AgentForUpdate 更新请求参数
///
/// # 返回
/// - `SchedAgent`: 更新后的 Agent 实体
///
/// # 错误
/// - Agent 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn update_agent(
  State(app): State<ServerApplication>,
  Path(agent_id): Path<Uuid>,
  Json(request): Json<AgentForUpdate>,
) -> WebResult<SchedAgent> {
  todo!()
}

/// 删除 Agent
///
/// 删除指定的 Agent，同时断开其连接
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `agent_id`: Agent 的唯一标识符
///
/// # 返回
/// - `()`: 删除操作结果
///
/// # 错误
/// - Agent 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn delete_agent(
  State(app): State<ServerApplication>,
  Path(agent_id): Path<Uuid>,
) -> WebResult<()> {
  todo!()
}

/// 获取 Agent 日志
///
/// 根据提供的查询条件获取指定 Agent 的日志记录
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `agent_id`: Agent 的唯一标识符
/// - `request`: 日志查询请求参数
///
/// # 返回
/// - `PageResult<String>`: 分页的 Agent 日志列表
///
/// # 错误
/// - Agent 不存在时返回 404 错误
/// - 数据库查询失败时返回相应错误
pub async fn get_agent_logs(
  State(app): State<ServerApplication>,
  Path(agent_id): Path<Uuid>,
  Json(request): Json<QueryRequest<LogFilter>>,
) -> WebResult<PageResult<String>> {
  todo!()
}

/// 获取 Agent 实时状态
///
/// 获取指定 Agent 的实时状态信息，包括连接状态和性能指标
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `agent_id`: Agent 的唯一标识符
///
/// # 返回
/// - `AgentStatusResponse`: Agent 状态信息
///
/// # 错误
/// - Agent 不存在时返回 404 错误
/// - 连接管理器查询失败时返回相应错误
pub async fn get_agent_status(
  State(app): State<ServerApplication>,
  Path(agent_id): Path<Uuid>,
) -> WebResult<AgentStatusResponse> {
  todo!()
}

/// 向 Agent 发送指令
///
/// 向指定的 Agent 发送控制指令
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `agent_id`: Agent 的唯一标识符
/// - `request`: 指令请求参数
///
/// # 返回
/// - `()`: 指令发送结果
///
/// # 错误
/// - Agent 不存在或离线时返回相应错误
/// - 消息发送失败时返回相应错误
pub async fn send_agent_command(
  State(app): State<ServerApplication>,
  Path(agent_id): Path<Uuid>,
  Json(request): Json<AgentCommandRequest>,
) -> WebResult<()> {
  let mm = &app.mm;
  // 验证 Agent 是否存在
  let _ = AgentBmc::get(mm, agent_id).await?;

  // 检查 Agent 是否在线
  let gateway = app.gateway_svc();
  let is_connected = gateway.connection_manager().is_agent_connected(&agent_id.to_string()).await;
  if !is_connected {
    return Err(ultimate_web::WebError::BadRequest("Agent is not connected".to_string()));
  }

  // 构建并发送 WebSocket 消息
  let message = WebSocketMessage {
    kind: MessageKind::Command,
    payload: request.parameters.unwrap_or_default(),
    timestamp:fusion_common::time::now_utc(),
  };

  gateway.connection_manager().send_to_agent(&agent_id.to_string(), message).await?;

  Ok(())
}
```

### 2.4 Job 管理 API 实现

```rust
// src/endpoint/jobs/mod.rs
pub mod handlers;

use axum::{routing::*, Router};
use std::sync::Arc;
use crate::ServerApplication;

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/query", post(handlers::query_jobs))
    .route("/create", post(handlers::create_job))
    .route("/:id", get(handlers::get_job).delete(handlers::delete_job))
    .route("/:id/update", post(handlers::update_job))
    .route("/:id/enable", post(handlers::enable_job))
    .route("/:id/disable", post(handlers::disable_job))
    .route("/namespace/:namespace", get(handlers::list_jobs_by_namespace))
}

// src/endpoint/jobs/handlers.rs
use axum::{
  extract::{Path, State, Query},
  Json,
  http::StatusCode,
};
use std::sync::Arc;
use crate::{
  ServerApplication,
  endpoint::types::*,
  model::job::*,
  infra::bmc::JobBmc,
};
use modelsql::{
  filter::page::Page,
  page::PageResult,
};
use fusion_web::WebResult;

/// 查询 Job 列表
///
/// 根据提供的过滤条件和分页参数查询 Job 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: 包含过滤条件和分页参数的查询请求
///
/// # 返回
/// - `PageResult<SchedJob>`: 分页的 Job 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn query_jobs(
  State(app): State<ServerApplication>,
  Json(request): Json<JobForQuery>,
) -> WebResult<PageResult<SchedJob>> {
  let mm = &app.mm;
  let result = JobBmc::list(mm, Some(request.filter), Some(request.page)).await?;
  Ok(result)
}

/// 创建新 Job
///
/// 根据提供的参数创建新的 Job
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: JobForCreate 创建请求参数
///
/// # 返回
/// - `SchedJob`: 创建的 Job 实体
///
/// # 错误
/// - 参数验证失败时返回相应错误
/// - 数据库操作失败时返回相应错误
pub async fn create_job(
  State(app): State<ServerApplication>,
  Json(request): Json<JobForCreate>,
) -> WebResult<SchedJob> {
  let mm = &app.mm;
  let job = JobBmc::insert(mm, request).await?;
  Ok(job)
}

/// 获取特定 Job 详情
///
/// 根据 Job ID 获取 Job 的详细信息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `job_id`: Job 的唯一标识符
///
/// # 返回
/// - `SchedJob`: Job 实体详情
///
/// # 错误
/// - Job 不存在时返回 404 错误
/// - 数据库查询失败时返回相应错误
pub async fn get_job(
  State(app): State<ServerApplication>,
  Path(job_id): Path<i64>,
) -> WebResult<SchedJob> {
  let mm = &app.mm;
  let job = JobBmc::get(mm, job_id).await?;
  Ok(job)
}

/// 更新 Job 信息
///
/// 根据提供的参数更新指定 Job 的信息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `job_id`: Job 的唯一标识符
/// - `request`: JobForUpdate 更新请求参数
///
/// # 返回
/// - `SchedJob`: 更新后的 Job 实体
///
/// # 错误
/// - Job 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn update_job(
  State(app): State<ServerApplication>,
  Path(job_id): Path<i64>,
  Json(request): Json<JobForUpdate>,
) -> WebResult<SchedJob> {
  let mm = &app.mm;
  let job = JobBmc::update(mm, job_id, request).await?;
  Ok(job)
}

/// 删除 Job
///
/// 删除指定的 Job
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `job_id`: Job 的唯一标识符
///
/// # 返回
/// - `()`: 删除操作结果
///
/// # 错误
/// - Job 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn delete_job(
  State(app): State<ServerApplication>,
  Path(job_id): Path<i64>,
) -> WebResult<()> {
  let mm = &app.mm;
  JobBmc::delete(mm, job_id).await?;
  Ok(())
}

/// 启用 Job
///
/// 启用指定的 Job，使其可以被调度执行
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `job_id`: Job 的唯一标识符
///
/// # 返回
/// - `SchedJob`: 更新后的 Job 实体
///
/// # 错误
/// - Job 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn enable_job(
  State(app): State<ServerApplication>,
  Path(job_id): Path<i64>,
) -> WebResult<SchedJob> {
  let mm = &app.mm;
  let update_data = JobForUpdate {
    enabled: Some(true),
    ..Default::default()
  };
  let job = JobBmc::update(mm, job_id, update_data).await?;
  Ok(job)
}

/// 禁用 Job
///
/// 禁用指定的 Job，停止其调度执行
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `job_id`: Job 的唯一标识符
///
/// # 返回
/// - `SchedJob`: 更新后的 Job 实体
///
/// # 错误
/// - Job 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn disable_job(
  State(app): State<ServerApplication>,
  Path(job_id): Path<i64>,
) -> WebResult<SchedJob> {
  let mm = &app.mm;
  let update_data = JobForUpdate {
    enabled: Some(false),
    ..Default::default()
  };
  let job = JobBmc::update(mm, job_id, update_data).await?;
  Ok(job)
}

/// 按命名空间获取 Job 列表
///
/// 根据命名空间获取 Job 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `namespace_id`: 命名空间名称
///
/// # 返回
/// - `Vec<SchedJob>`: Job 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn list_jobs_by_namespace(
  State(app): State<ServerApplication>,
  Path(namespace_id): Path<String>,
) -> WebResult<Vec<SchedJob>> {
  let mm = &app.model_manager;
  let filter = JobFilter {
    namespace_id: Some(namespace_id),
    ..Default::default()
  };
  let result = JobBmc::list(mm, Some(filter), None).await?;
  Ok(result.data)
}
```

### 2.5 Task 管理 API 实现

```rust
// src/endpoint/tasks/mod.rs
pub mod handlers;

use axum::{routing::*, Router};
use std::sync::Arc;
use crate::ServerApplication;

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/query", post(handlers::query_tasks))
    .route("/create", post(handlers::create_task))
    .route("/:id", get(handlers::get_task).delete(handlers::delete_task))
    .route("/:id/update", post(handlers::update_task))
    .route("/:id/retry", post(handlers::retry_task))
    .route("/:id/cancel", post(handlers::cancel_task))
    .route("/job/:job_id", get(handlers::list_tasks_by_job))
    .route("/agent/:agent_id", get(handlers::list_tasks_by_agent))
    .route("/status/:status", get(handlers::list_tasks_by_status))
}

// src/endpoint/tasks/handlers.rs
use axum::{
  extract::{Path, State, Query},
  Json,
  http::StatusCode,
};
use std::sync::Arc;
use crate::{
  ServerApplication,
  endpoint::types::*,
  model::task::*,
  infra::bmc::TaskBmc,
};
use modelsql::{
  filter::page::Page,
  page::PageResult,
};
use fusion_web::WebResult;

/// 查询 Task 列表
///
/// 根据提供的过滤条件和分页参数查询 Task 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: 包含过滤条件和分页参数的查询请求
///
/// # 返回
/// - `PageResult<SchedTask>`: 分页的 Task 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn query_tasks(
  State(app): State<ServerApplication>,
  Json(request): Json<TaskForQuery>,
) -> WebResult<PageResult<SchedTask>> {
  let mm = &app.mm;
  let result = TaskBmc::list(mm, Some(request.filter), Some(request.page)).await?;
  Ok(result)
}

/// 创建新 Task
///
/// 根据提供的参数创建新的 Task
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: TaskForCreate 创建请求参数
///
/// # 返回
/// - `SchedTask`: 创建的 Task 实体
///
/// # 错误
/// - 参数验证失败时返回相应错误
/// - 数据库操作失败时返回相应错误
pub async fn create_task(
  State(app): State<ServerApplication>,
  Json(request): Json<TaskForCreate>,
) -> WebResult<SchedTask> {
  let mm = &app.mm;
  let task = TaskBmc::insert(mm, request).await?;
  Ok(task)
}

/// 获取特定 Task 详情
///
/// 根据 Task ID 获取 Task 的详细信息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `task_id`: Task 的唯一标识符
///
/// # 返回
/// - `SchedTask`: Task 实体详情
///
/// # 错误
/// - Task 不存在时返回 404 错误
/// - 数据库查询失败时返回相应错误
pub async fn get_task(
  State(app): State<ServerApplication>,
  Path(task_id): Path<i64>,
) -> WebResult<SchedTask> {
  let mm = &app.mm;
  let task = TaskBmc::get(mm, task_id).await?;
  Ok(task)
}

/// 更新 Task 信息
///
/// 根据提供的参数更新指定 Task 的信息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `task_id`: Task 的唯一标识符
/// - `request`: TaskForUpdate 更新请求参数
///
/// # 返回
/// - `SchedTask`: 更新后的 Task 实体
///
/// # 错误
/// - Task 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn update_task(
  State(app): State<ServerApplication>,
  Path(task_id): Path<i64>,
  Json(request): Json<TaskForUpdate>,
) -> WebResult<SchedTask> {
  let mm = &app.mm;
  let task = TaskBmc::update(mm, task_id, request).await?;
  Ok(task)
}

/// 删除 Task
///
/// 删除指定的 Task
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `task_id`: Task 的唯一标识符
///
/// # 返回
/// - `()`: 删除操作结果
///
/// # 错误
/// - Task 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn delete_task(
  State(app): State<ServerApplication>,
  Path(task_id): Path<i64>,
) -> WebResult<()> {
  let mm = &app.mm;
  TaskBmc::delete(mm, task_id).await?;
  Ok(())
}

/// 按 Job 获取 Task 列表
///
/// 根据 Job ID 获取 Task 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `job_id`: Job 的唯一标识符
///
/// # 返回
/// - `Vec<SchedTask>`: Task 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn list_tasks_by_job(
  State(app): State<ServerApplication>,
  Path(job_id): Path<i64>,
) -> WebResult<Vec<SchedTask>> {
  let mm = &app.model_manager;
  let filter = TaskFilter {
    job_id: Some(job_id),
    ..Default::default()
  };
  let result = TaskBmc::list(mm, Some(filter), None).await?;
  Ok(result.data)
}

/// 按 Agent 获取 Task 列表
///
/// 根据 Agent ID 获取 Task 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `agent_id`: Agent 的唯一标识符
///
/// # 返回
/// - `Vec<SchedTask>`: Task 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn list_tasks_by_agent(
  State(app): State<ServerApplication>,
  Path(agent_id): Path<i64>,
) -> WebResult<Vec<SchedTask>> {
  let mm = &app.model_manager;
  let filter = TaskFilter {
    agent_id: Some(agent_id),
    ..Default::default()
  };
  let result = TaskBmc::list(mm, Some(filter), None).await?;
  Ok(result.data)
}

/// 按状态获取 Task 列表
///
/// 根据状态获取 Task 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `status`: Task 状态
///
/// # 返回
/// - `Vec<SchedTask>`: Task 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn list_tasks_by_status(
  State(app): State<ServerApplication>,
  Path(status): Path<String>,
) -> WebResult<Vec<SchedTask>> {
  let mm = &app.model_manager;
  let filter = TaskFilter {
    status: Some(status),
    ..Default::default()
  };
  let result = TaskBmc::list(mm, Some(filter), None).await?;
  Ok(result.data)
}

/// 重试 Task
///
/// 重新执行指定的 Task
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `task_id`: Task 的唯一标识符
///
/// # 返回
/// - `SchedTask`: 更新后的 Task 实体
///
/// # 错误
/// - Task 不存在时返回 404 错误
/// - Task 状态不允许重试时返回相应错误
pub async fn retry_task(
  State(app): State<ServerApplication>,
  Path(task_id): Path<i64>,
) -> WebResult<SchedTask> {
  let mm = &app.model_manager;

  // 重置任务状态
  let update_data = TaskForUpdate {
    status: Some("pending".to_string()),
    ..Default::default()
  };
  let task = TaskBmc::update(mm, task_id, update_data).await?;

  // TODO: 通过调度器重新调度任务
  if let Some(scheduler) = &app.scheduler_svc {
    // scheduler.schedule_task(task_id).await?;
  }

  Ok(task)
}

/// 取消 Task
///
/// 取消指定的 Task 执行
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `task_id`: Task 的唯一标识符
///
/// # 返回
/// - `SchedTask`: 更新后的 Task 实体
///
/// # 错误
/// - Task 不存在时返回 404 错误
/// - Task 状态不允许取消时返回相应错误
pub async fn cancel_task(
  State(app): State<ServerApplication>,
  Path(task_id): Path<i64>,
) -> WebResult<SchedTask> {
  let mm = &app.model_manager;

  let update_data = TaskForUpdate {
    status: Some("cancelled".to_string()),
    ..Default::default()
  };
  let task = TaskBmc::update(mm, task_id, update_data).await?;

  // TODO: 通过调度器取消任务
  if let Some(scheduler) = &app.scheduler_svc {
    // scheduler.cancel_task(task_id).await?;
  }

  Ok(task)
}
```

### 3.5 TaskInstance 管理 API 实现

基于实际的 TaskInstance 数据模型：

```rust
// src/endpoint/task_instances/mod.rs
pub mod handlers;

use axum::{routing::*, Router};
use std::sync::Arc;
use crate::ServerApplication;

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/query", post(handlers::query_task_instances))
    .route("/create", post(handlers::create_task_instance))
    .route("/:id", get(handlers::get_task_instance).delete(handlers::delete_task_instance))
    .route("/:id/update", post(handlers::update_task_instance))
    .route("/:id/start", post(handlers::start_task_instance))
    .route("/:id/complete", post(handlers::complete_task_instance))
    .route("/:id/cancel", post(handlers::cancel_task_instance))
    .route("/task/:task_id", get(handlers::list_task_instances_by_task))
    .route("/agent/:agent_id", get(handlers::list_task_instances_by_agent))
    .route("/status/:status", get(handlers::list_task_instances_by_status))
}

// src/endpoint/task_instances/handlers.rs
use axum::{
  extract::{Path, State},
  Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
  ServerApplication,
  endpoint::types::*,
  model::task_instance::{SchedTaskInstance, TaskInstanceForCreate, TaskInstanceForUpdate, TaskInstanceFilter},
  infra::bmc::TaskInstanceBmc,
};
use modelsql::{
  filter::page::Page,
  page::PageResult,
};
use fusion_web::WebResult;

/// 查询 TaskInstance 列表
///
/// 根据提供的过滤条件和分页参数查询 TaskInstance 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: 包含过滤条件和分页参数的查询请求
///
/// # 返回
/// - `PageResult<SchedTaskInstance>`: 分页的 TaskInstance 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn query_task_instances(
  State(app): State<ServerApplication>,
  Json(request): Json<TaskInstanceForQuery>,
) -> WebResult<PageResult<SchedTaskInstance>> {
  let mm = &app.mm;
  let result = TaskInstanceBmc::list(mm, Some(request.filter), Some(request.page)).await?;
  Ok(result)
}

/// 创建新 TaskInstance
///
/// 根据提供的参数创建新的 TaskInstance
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: TaskInstanceForCreate 创建请求参数
///
/// # 返回
/// - `SchedTaskInstance`: 创建的 TaskInstance 实体
///
/// # 错误
/// - 参数验证失败时返回相应错误
/// - 数据库操作失败时返回相应错误
pub async fn create_task_instance(
  State(app): State<ServerApplication>,
  Json(request): Json<TaskInstanceForCreate>,
) -> WebResult<SchedTaskInstance> {
  let mm = &app.mm;
  let task_instance = TaskInstanceBmc::insert(mm, request).await?;
  Ok(task_instance)
}

/// 获取特定 TaskInstance 详情
///
/// 根据 TaskInstance ID 获取 TaskInstance 的详细信息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `instance_id`: TaskInstance 的唯一标识符
///
/// # 返回
/// - `SchedTaskInstance`: TaskInstance 实体详情
///
/// # 错误
/// - TaskInstance 不存在时返回 404 错误
/// - 数据库查询失败时返回相应错误
pub async fn get_task_instance(
  State(app): State<ServerApplication>,
  Path(instance_id): Path<Uuid>,
) -> WebResult<SchedTaskInstance> {
  let mm = &app.mm;
  let task_instance = TaskInstanceBmc::get(mm, instance_id).await?;
  Ok(task_instance)
}

/// 更新 TaskInstance 信息
///
/// 根据提供的参数更新指定 TaskInstance 的信息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `instance_id`: TaskInstance 的唯一标识符
/// - `request`: TaskInstanceForUpdate 更新请求参数
///
/// # 返回
/// - `SchedTaskInstance`: 更新后的 TaskInstance 实体
///
/// # 错误
/// - TaskInstance 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn update_task_instance(
  State(app): State<ServerApplication>,
  Path(instance_id): Path<Uuid>,
  Json(request): Json<TaskInstanceForUpdate>,
) -> WebResult<SchedTaskInstance> {
  let mm = &app.mm;
  let task_instance = TaskInstanceBmc::update(mm, instance_id, request).await?;
  Ok(task_instance)
}

/// 删除 TaskInstance
///
/// 删除指定的 TaskInstance
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `instance_id`: TaskInstance 的唯一标识符
///
/// # 返回
/// - `()`: 删除操作结果
///
/// # 错误
/// - TaskInstance 不存在时返回 404 错误
/// - 数据库操作失败时返回相应错误
pub async fn delete_task_instance(
  State(app): State<ServerApplication>,
  Path(instance_id): Path<Uuid>,
) -> WebResult<()> {
  let mm = &app.mm;
  TaskInstanceBmc::delete(mm, instance_id).await?;
  Ok(())
}

/// 启动 TaskInstance
///
/// 启动指定的 TaskInstance 执行
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `instance_id`: TaskInstance 的唯一标识符
/// - `request`: StartTaskInstanceRequest 启动请求参数
///
/// # 返回
/// - `SchedTaskInstance`: 更新后的 TaskInstance 实体
///
/// # 错误
/// - TaskInstance 不存在时返回 404 错误
/// - TaskInstance 状态不允许启动时返回相应错误
pub async fn start_task_instance(
  State(app): State<ServerApplication>,
  Path(instance_id): Path<Uuid>,
  Json(request): Json<StartTaskInstanceRequest>,
) -> WebResult<SchedTaskInstance> {
  let mm = &app.mm;

  // 更新任务实例状态为运行中
  let update_data = TaskInstanceForUpdate {
    status: Some("running".to_string()),
    started_at: Some(ultimate_common::time::now_utc()),
    ..Default::default()
  };
  let task_instance = TaskInstanceBmc::update(mm, instance_id, update_data).await?;

  // TODO: 通过调度器启动任务实例
  if let Some(scheduler) = &app.scheduler_svc {
    // scheduler.start_task_instance(instance_id).await?;
  }

  Ok(task_instance)
}

/// 完成 TaskInstance
///
/// 标记指定的 TaskInstance 为完成状态
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `instance_id`: TaskInstance 的唯一标识符
/// - `request`: CompleteTaskInstanceRequest 完成请求参数
///
/// # 返回
/// - `SchedTaskInstance`: 更新后的 TaskInstance 实体
///
/// # 错误
/// - TaskInstance 不存在时返回 404 错误
/// - TaskInstance 状态不允许完成时返回相应错误
pub async fn complete_task_instance(
  State(app): State<ServerApplication>,
  Path(instance_id): Path<Uuid>,
  Json(request): Json<CompleteTaskInstanceRequest>,
) -> WebResult<SchedTaskInstance> {
  let mm = &app.mm;

  let update_data = TaskInstanceForUpdate {
    status: Some("completed".to_string()),
    completed_at: Some(ultimate_common::time::now_utc()),
    result: request.result,
    ..Default::default()
  };
  let task_instance = TaskInstanceBmc::update(mm, instance_id, update_data).await?;

  Ok(task_instance)
}

/// 取消 TaskInstance
///
/// 取消指定的 TaskInstance 执行
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `instance_id`: TaskInstance 的唯一标识符
///
/// # 返回
/// - `SchedTaskInstance`: 更新后的 TaskInstance 实体
///
/// # 错误
/// - TaskInstance 不存在时返回 404 错误
/// - TaskInstance 状态不允许取消时返回相应错误
pub async fn cancel_task_instance(
  State(app): State<ServerApplication>,
  Path(instance_id): Path<Uuid>,
) -> WebResult<SchedTaskInstance> {
  let mm = &app.mm;

  let update_data = TaskInstanceForUpdate {
    status: Some("cancelled".to_string()),
    completed_at: Some(ultimate_common::time::now_utc()),
    ..Default::default()
  };
  let task_instance = TaskInstanceBmc::update(mm, instance_id, update_data).await?;

  // TODO: 通过调度器取消任务实例
  if let Some(scheduler) = &app.scheduler_svc {
    // scheduler.cancel_task_instance(instance_id).await?;
  }

  Ok(task_instance)
}

/// 按 Task 获取 TaskInstance 列表
///
/// 根据 Task ID 获取 TaskInstance 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `task_id`: Task 的唯一标识符
///
/// # 返回
/// - `Vec<SchedTaskInstance>`: TaskInstance 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn list_task_instances_by_task(
  State(app): State<ServerApplication>,
  Path(task_id): Path<i64>,
) -> WebResult<Vec<SchedTaskInstance>> {
  let mm = &app.mm;
  let filter = TaskInstanceFilter {
    task_id: Some(task_id),
    ..Default::default()
  };
  let result = TaskInstanceBmc::list(mm, Some(filter), None).await?;
  Ok(result.data)
}

/// 按 Agent 获取 TaskInstance 列表
///
/// 根据 Agent ID 获取 TaskInstance 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `agent_id`: Agent 的唯一标识符
///
/// # 返回
/// - `Vec<SchedTaskInstance>`: TaskInstance 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn list_task_instances_by_agent(
  State(app): State<ServerApplication>,
  Path(agent_id): Path<Uuid>,
) -> WebResult<Vec<SchedTaskInstance>> {
  let mm = &app.mm;
  let filter = TaskInstanceFilter {
    agent_id: Some(agent_id),
    ..Default::default()
  };
  let result = TaskInstanceBmc::list(mm, Some(filter), None).await?;
  Ok(result.data)
}

/// 按状态获取 TaskInstance 列表
///
/// 根据状态获取 TaskInstance 列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `status`: TaskInstance 状态
///
/// # 返回
/// - `Vec<SchedTaskInstance>`: TaskInstance 列表
///
/// # 错误
/// - 数据库查询失败时返回相应错误
pub async fn list_task_instances_by_status(
  State(app): State<ServerApplication>,
  Path(status): Path<String>,
) -> WebResult<Vec<SchedTaskInstance>> {
  let mm = &app.mm;
  let filter = TaskInstanceFilter {
    status: Some(status),
    ..Default::default()
  };
  let result = TaskInstanceBmc::list(mm, Some(filter), None).await?;
  Ok(result.data)
}
```

### 2.10 连接管理 API 实现

```rust
// src/endpoint/connections/mod.rs
pub mod handlers;

use axum::{routing::*, Router};
use crate::ServerApplication;

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/query", post(handlers::query_connections))
    .route("/stats", post(handlers::get_connection_stats))
    .route("/:agent_id", delete(handlers::disconnect_agent))
    .route("/broadcast", post(handlers::broadcast_message))
}

// src/endpoint/connections/handlers.rs
use axum::{
  extract::{Path, State},
  Json,
};
use uuid::Uuid;
use crate::{
  ServerApplication,
  endpoint::types::*,
};
use fusion_web::WebResult;
use modelsql::page::PageResult;

/// 查询连接列表
///
/// 获取当前活跃的 Agent 连接列表
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: 包含过滤条件和分页参数的查询请求
///
/// # 返回
/// - `PageResult<ConnectionInfo>`: 分页的连接信息列表
///
/// # 错误
/// - 连接管理器查询失败时返回相应错误
pub async fn query_connections(
  State(app): State<ServerApplication>,
  Json(request): Json<QueryRequest<ConnectionFilter>>,
) -> WebResult<PageResult<ConnectionInfo>> {
  let gateway = app.gateway_svc();
  let connections = gateway.connection_manager().query_connections(request.filter, request.page).await?;
  Ok(connections)
}

/// 获取连接统计信息
///
/// 获取系统的连接统计数据
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: ConnectionStatsRequest 统计请求参数
///
/// # 返回
/// - `ConnectionStats`: 连接统计信息
///
/// # 错误
/// - 连接管理器查询失败时返回相应错误
pub async fn get_connection_stats(
  State(app): State<ServerApplication>,
  Json(request): Json<ConnectionStatsRequest>,
) -> WebResult<ConnectionStats> {
  let gateway = app.gateway_svc();
  let stats = gateway.connection_manager().get_connection_stats(request).await?;
  Ok(stats)
}

/// 断开 Agent 连接
///
/// 强制断开指定 Agent 的连接
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `agent_id`: Agent 的唯一标识符
///
/// # 返回
/// - `()`: 断开连接操作结果
///
/// # 错误
/// - Agent 不存在或未连接时返回相应错误
/// - 连接管理器操作失败时返回相应错误
pub async fn disconnect_agent(
  State(app): State<ServerApplication>,
  Path(agent_id): Path<Uuid>,
) -> WebResult<()> {
  let gateway = app.gateway_svc();
  gateway.connection_manager().disconnect_agent(agent_id).await?;
  Ok(())
}

/// 广播消息
///
/// 向所有在线 Agent 广播消息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: BroadcastMessageRequest 广播请求参数
///
/// # 返回
/// - `BroadcastResult`: 广播操作结果
///
/// # 错误
/// - 消息格式错误时返回相应错误
/// - 连接管理器操作失败时返回相应错误
pub async fn broadcast_message(
  State(app): State<ServerApplication>,
  Json(request): Json<BroadcastMessageRequest>,
) -> WebResult<BroadcastResult> {
  let gateway = app.gateway_svc();
  let result = gateway.connection_manager().broadcast_message(request).await?;
  Ok(result)
}
```

### 2.11 连接管理相关数据结构

```rust
// src/model/connection.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use fusion_common::time::OffsetDateTime;
use hetuflow_core::{
  types::AgentStatus,
  protocol::MessageKind,
};
// 已移除 PaginationQuery，使用 Page 替代

/// 连接查询请求
#[derive(Debug, Deserialize)]
pub struct ConnectionForQuery {
  /// Agent 状态过滤
  pub status: Option<AgentStatus>,
  /// 连接时长过滤（秒）
  pub min_duration: Option<u64>,
  /// 分页参数
  pub pagination: Page,
}

/// 连接统计请求
#[derive(Debug, Deserialize)]
pub struct ConnectionStatsRequest {
  /// 是否包含详细统计
  pub include_details: Option<bool>,
}

/// 广播消息请求
#[derive(Debug, Deserialize)]
pub struct BroadcastMessageRequest {
  /// 消息类型
  pub message_kind: MessageKind,
  /// 消息载荷
  pub payload: serde_json::Value,
  /// 目标 Agent 过滤（可选）
  pub target_agents: Option<Vec<Uuid>>,
  /// 消息优先级
  pub priority: Option<u8>,
}

/// 连接信息
#[derive(Debug, Serialize)]
pub struct ConnectionInfo {
  /// Agent ID
  pub agent_id: Uuid,
  /// 连接状态
  pub status: AgentStatus,
  /// 连接建立时间
  pub connected_at: OffsetDateTime,
  /// 最后活动时间
  pub last_activity: OffsetDateTime,
  /// 连接时长（秒）
  pub duration: u64,
  /// 发送消息数量
  pub messages_sent: u64,
  /// 接收消息数量
  pub messages_received: u64,
}

/// 连接统计信息
#[derive(Debug, Serialize)]
pub struct ConnectionStats {
  /// 总连接数
  pub total_connections: u32,
  /// 在线连接数
  pub online_connections: u32,
  /// 空闲连接数
  pub idle_connections: u32,
  /// 忙碌连接数
  pub busy_connections: u32,
  /// 平均连接时长（秒）
  pub average_duration: f64,
  /// 总发送消息数
  pub total_messages_sent: u64,
  /// 总接收消息数
  pub total_messages_received: u64,
}

/// 广播结果
#[derive(Debug, Serialize)]
pub struct BroadcastResult {
  /// 目标 Agent 数量
  pub target_count: u32,
  /// 成功发送数量
  pub success_count: u32,
  /// 失败发送数量
  pub failed_count: u32,
  /// 失败的 Agent ID 列表
  pub failed_agents: Vec<Uuid>,
}
```

### 2.12 系统监控 API 实现

```rust
// src/endpoint/system/mod.rs
pub mod handlers;

use axum::{routing::*, Router};
use crate::ServerApplication;

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .route("/health", get(handlers::health_check))
    .route("/metrics", post(handlers::get_metrics))
    .route("/info", post(handlers::get_system_info))
}

// src/endpoint/system/handlers.rs
use axum::{
  extract::State,
  Json,
};
use crate::{
  ServerApplication,
  endpoint::types::*,
};
use fusion_web::WebResult;
use serde::Serialize;

/// 健康检查
///
/// 检查系统的整体健康状态，包括数据库连接和基础服务
///
/// # 参数
/// - `app`: ServerApplication 应用实例
///
/// # 返回
/// - `HealthStatus`: 系统健康状态信息
///
/// # 错误
/// - 系统检查失败时返回相应错误
pub async fn health_check(
  State(app): State<ServerApplication>,
) -> WebResult<HealthStatus> {
  let mm = app.mm;
  let gateway = app.gateway_svc();

  // 检查数据库连接
  let db_status = mm.db().ping().await.is_ok();

  // 检查连接管理器状态
  let connection_count = gateway.connection_manager().get_connection_count().await;

  let health_status = HealthStatus {
    status: if db_status { "healthy" } else { "unhealthy" }.to_string(),
    database: db_status,
    connections: connection_count,
    timestamp:fusion_common::time::now_utc(),
  };

  Ok(health_status)
}

/// 获取系统指标
///
/// 获取系统的性能指标和统计信息
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: MetricsRequest 指标请求参数
///
/// # 返回
/// - `SystemMetrics`: 系统指标信息
///
/// # 错误
/// - 指标收集失败时返回相应错误
pub async fn get_metrics(
  State(app): State<ServerApplication>,
  Json(request): Json<MetricsRequest>,
) -> WebResult<SystemMetrics> {
  let mm = app.mm;
  let gateway = app.gateway_svc();

  // 收集连接统计信息
  let connection_stats = gateway.connection_manager().get_connection_stats(ConnectionStatsRequest {
    include_details: false,
  }).await?;

  // 收集任务执行统计
  let task_stats = TaskInstanceBmc::get_execution_stats(mm).await?;

  let metrics = SystemMetrics {
    connections: connection_stats,
    tasks: task_stats,
    timestamp:fusion_common::time::now_utc(),
  };

  Ok(metrics)
}

/// 获取系统信息
///
/// 获取系统的基本信息，如版本、构建信息等
///
/// # 参数
/// - `app`: ServerApplication 应用实例
/// - `request`: SystemInfoRequest 系统信息请求参数
///
/// # 返回
/// - `SystemInfo`: 系统基本信息
///
/// # 错误
/// - 信息获取失败时返回相应错误
pub async fn get_system_info(
  State(app): State<ServerApplication>,
  Json(request): Json<SystemInfoRequest>,
) -> WebResult<SystemInfo> {
  let system_info = SystemInfo {
    name: "Hetuflow Server".to_string(),
    version: env!("CARGO_PKG_VERSION").to_string(),
    build_time: env!("BUILD_TIME").to_string(),
    git_commit: env!("GIT_COMMIT").to_string(),
    is_leader: app.is_leader(),
    uptime: app.get_uptime().as_secs(),
    timestamp:fusion_common::time::now_utc(),
  };

  Ok(system_info)
}
```

### 2.13 系统监控相关数据结构

```rust
// src/model/metrics.rs
use serde::{Deserialize, Serialize};
use fusion_common::time::OffsetDateTime;

use hetuflow_core::models::ConnectionStats;

/// 指标请求
#[derive(Debug, Deserialize)]
pub struct MetricsRequest {
  /// 是否包含详细指标
  pub include_details: Option<bool>,
  /// 指标时间范围（分钟）
  pub time_range: Option<u32>,
}

/// 系统信息请求
#[derive(Debug, Deserialize)]
pub struct SystemInfoRequest {
  /// 是否包含配置信息
  pub include_config: Option<bool>,
  /// 是否包含环境信息
  pub include_environment: Option<bool>,
}

/// 健康状态
#[derive(Debug, Serialize)]
pub struct HealthStatus {
  /// 整体状态
  pub status: String,
  /// 检查时间戳
  pub timestamp: OffsetDateTime,
  /// 数据库健康状态
  pub database: bool,
  /// 活跃连接数
  pub connections: u32,
}

/// 健康检查详细信息
#[derive(Debug, Serialize)]
pub struct HealthDetails {
  /// 数据库连接池状态
  pub database_pool: DatabasePoolStatus,
  /// 各服务状态
  pub services: Vec<ServiceStatus>,
  /// 资源使用情况
  pub resources: ResourceUsage,
}

/// 数据库连接池状态
#[derive(Debug, Serialize)]
pub struct DatabasePoolStatus {
  /// 活跃连接数
  pub active_connections: u32,
  /// 空闲连接数
  pub idle_connections: u32,
  /// 最大连接数
  pub max_connections: u32,
  /// 等待连接数
  pub waiting_connections: u32,
}

/// 服务状态
#[derive(Debug, Serialize)]
pub struct ServiceStatus {
  /// 服务名称
  pub name: String,
  /// 服务状态
  pub status: String,
  /// 最后检查时间
  pub last_check: OffsetDateTime,
  /// 响应时间（毫秒）
  pub response_time: u64,
}

/// 资源使用情况
#[derive(Debug, Serialize)]
pub struct ResourceUsage {
  /// CPU 使用率
  pub cpu_usage: f32,
  /// 内存使用率
  pub memory_usage: f32,
  /// 磁盘使用率
  pub disk_usage: f32,
  /// 网络 I/O
  pub network_io: NetworkIO,
}

/// 网络 I/O 统计
#[derive(Debug, Serialize)]
pub struct NetworkIO {
  /// 接收字节数
  pub bytes_received: u64,
  /// 发送字节数
  pub bytes_sent: u64,
  /// 接收包数
  pub packets_received: u64,
  /// 发送包数
  pub packets_sent: u64,
}

/// 系统指标
#[derive(Debug, Serialize)]
pub struct SystemMetrics {
  /// 连接统计
  pub connections: ConnectionStats,
  /// 任务统计
  pub tasks: TaskExecutionStats,
  /// 时间戳
  pub timestamp: OffsetDateTime,
}

/// 任务执行统计
#[derive(Debug, Serialize)]
pub struct TaskExecutionStats {
  /// 总任务数
  pub total_tasks: u64,
  /// 运行中任务数
  pub running_tasks: u64,
  /// 已完成任务数
  pub completed_tasks: u64,
  /// 失败任务数
  pub failed_tasks: u64,
}

/// 系统信息
#[derive(Debug, Serialize)]
pub struct SystemInfo {
  /// 应用名称
  pub name: String,
  /// 版本号
  pub version: String,
  /// 构建时间
  pub build_time: String,
  /// Git 提交哈希
  pub git_commit: String,
  /// 是否为领导节点
  pub is_leader: bool,
  /// 运行时间（秒）
  pub uptime: u64,
  /// 时间戳
  pub timestamp: OffsetDateTime,
  /// 连接统计
  pub connections: ConnectionStats,
  /// 任务统计
  pub tasks: TaskStats,
  /// 系统资源
  pub resources: ResourceUsage,
  /// 数据库指标
  pub database: DatabaseMetrics,
  /// 应用指标
  pub application: ApplicationMetrics,
}

/// 任务统计
#[derive(Debug, Serialize)]
pub struct TaskStats {
  /// 总任务数
  pub total_tasks: u32,
  /// 运行中任务数
  pub running_tasks: u32,
  /// 等待中任务数
  pub pending_tasks: u32,
  /// 已完成任务数
  pub completed_tasks: u32,
  /// 失败任务数
  pub failed_tasks: u32,
  /// 平均执行时间（秒）
  pub average_execution_time: f64,
}

/// 数据库指标
#[derive(Debug, Serialize)]
pub struct DatabaseMetrics {
  /// 查询总数
  pub total_queries: u64,
  /// 平均查询时间（毫秒）
  pub average_query_time: f64,
  /// 慢查询数
  pub slow_queries: u32,
  /// 连接池使用率
  pub pool_usage: f32,
}

/// 应用指标
#[derive(Debug, Serialize)]
pub struct ApplicationMetrics {
  /// 启动时间
  pub startup_time: OffsetDateTime,
  /// 运行时长（秒）
  pub uptime: u64,
  /// 版本信息
  pub version: String,
  /// 构建信息
  pub build_info: BuildInfo,
}

/// 构建信息
#[derive(Debug, Serialize)]
pub struct BuildInfo {
  /// 构建版本
  pub version: String,
  /// 构建时间
  pub build_time: OffsetDateTime,
  /// 构建环境
  pub build_environment: String,
  /// Git 提交哈希
  pub git_commit: Option<String>,
  /// Git 分支
  pub git_branch: Option<String>,
}

/// 系统信息
#[derive(Debug, Serialize)]
pub struct SystemInfo {
  /// 应用名称
  pub name: String,
  /// 应用版本
  pub version: String,
  /// 构建信息
  pub build: BuildInfo,
  /// 运行环境
  pub environment: EnvironmentInfo,
  /// 支持的功能
  pub features: Vec<String>,
  /// 配置信息（可选）
  pub config: Option<ConfigInfo>,
}

/// 环境信息
#[derive(Debug, Serialize)]
pub struct EnvironmentInfo {
  /// 操作系统
  pub os: String,
  /// 架构
  pub architecture: String,
  /// Rust 版本
  pub rust_version: String,
  /// 主机名
  pub hostname: String,
}

/// 配置信息
#[derive(Debug, Serialize)]
pub struct ConfigInfo {
  /// 数据库类型
  pub database_type: String,
  /// WebSocket 端口
  pub websocket_port: u16,
  /// HTTP 端口
  pub http_port: u16,
  /// 日志级别
  pub log_level: String,
  /// 时区
  pub timezone: String,
}
```
