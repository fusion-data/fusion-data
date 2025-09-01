# Hetumind API 设计

## 1. API 架构概述

Hetumind API 基于 Axum 框架构建，提供 RESTful API 和 WebSocket 实时通信，支持现代化的认证和授权机制。

### 1.1 设计原则

- **RESTful 设计**: 遵循 REST 架构原则
- **类型安全**: 利用 Rust 类型系统确保 API 安全
- **异步优先**: 全面采用 async/await 模式
- **版本控制**: 支持 API 版本管理
- **文档化**: 自动生成 OpenAPI 文档

## 2. API 结构设计

### 2.1 路由架构

根路由定义在 [init_web](../../../crates/hetumind/hetumind-studio/src/endpoint/mod.rs) 函数。

### 2.2 应用状态

- [Hetumind](../../../crates/hetumind/hetumind-context/src/hetumind.rs)

## 3. 工作流 API

### 3.1 工作流路由

```rust
pub fn routes() -> Router<Hetumind> {
    Router::new()
        .route("/", post(create_workflow))
        .route("/query", post(query_workflows))
        .route("/validate", post(validate_workflow))
        .route("/:id", get(get_workflow).put(update_workflow).delete(delete_workflow))
        .route("/:id/execute", post(execute_workflow))
        .route("/:id/activate", post(activate_workflow))
        .route("/:id/deactivate", post(deactivate_workflow))
        .route("/:id/duplicate", post(duplicate_workflow))
}
```

### 3.2 工作流 API 实现

````rust
use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
};
use hetumind_context::hetumind::Hetumind;
use hetumind_core::{
    common::ParameterMap,
    workflow::{ExecutionId, Workflow, WorkflowId},
};
use modelsql::page::PageResult;
use serde::{Deserialize, Serialize};
use fusion_web::{WebError, WebResult};

use crate::domain::workflow::{WorkflowForQuery, WorkflowForUpdate, WorkflowSvc};

// --- API Data Models ---

/// The standard response for a single workflow.
/// It uses the core workflow model directly.
pub type WorkflowResponse = Workflow;

/// Request body for creating a new workflow.
/// It uses the core workflow model directly.
pub type CreateWorkflowRequest = Workflow;

/// Request body for validating a workflow.
/// It uses the core workflow model directly.
pub type ValidateWorkflowRequest = Workflow;

#[derive(Debug, Serialize)]
pub struct ValidateWorkflowResponse {
    pub is_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub input_data: Option<ParameterMap>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionIdResponse {
    pub execution_id: ExecutionId,
}

// --- API Handlers ---

/// 列出工作流
pub async fn query_workflows(
    workflow_svc: WorkflowSvc,
    Json(input): Json<WorkflowForQuery>,
) -> WebResult<PageResult<WorkflowResponse>> {
    todo!()
}

/// 验证工作流定义
pub async fn validate_workflow(
    workflow_svc: WorkflowSvc,
    Json(input): Json<ValidateWorkflowRequest>,
) -> WebResult<ValidateWorkflowResponse> {
    todo!()
}

/// 创建或导入工作流
pub async fn create_workflow(
    workflow_svc: WorkflowSvc,
    Json(input): Json<CreateWorkflowRequest>,
) -> WebResult<WorkflowResponse> {
    todo!()
}

/// 获取工作流详情
pub async fn get_workflow(
    workflow_svc: WorkflowSvc,
    Path(workflow_id): Path<WorkflowId>,
) -> WebResult<WorkflowResponse> {
    todo!()
}

/// 更新工作流
pub async fn update_workflow(
    workflow_svc: WorkflowSvc,
    Path(workflow_id): Path<WorkflowId>,
    Json(input): Json<WorkflowForUpdate>,
) -> WebResult<WorkflowResponse> {
    todo!()
}

/// 删除工作流
pub async fn delete_workflow(
    workflow_svc: WorkflowSvc,
    Path(workflow_id): Path<WorkflowId>,
) -> WebResult<()> {
    todo!()
}

/// 执行工作流
pub async fn execute_workflow(
    workflow_svc: WorkflowSvc,
    Path(workflow_id): Path<WorkflowId>,
    Json(input): Json<ExecuteWorkflowRequest>,
) -> WebResult<ExecutionIdResponse> {
    todo!()
}

/// 激活工作流
pub async fn activate_workflow(
    workflow_svc: WorkflowSvc,
    Path(workflow_id): Path<WorkflowId>,
) -> WebResult<()> {
    todo!()
}

/// 停用工作流
pub async fn deactivate_workflow(
    workflow_svc: WorkflowSvc,
    Path(workflow_id): Path<WorkflowId>,
) -> WebResult<()> {
    todo!()
}

/// 复制工作流
pub async fn duplicate_workflow(
    workflow_svc: WorkflowSvc,
    Path(workflow_id): Path<WorkflowId>,
) -> WebResult<()> {
    todo!()
}

# # 4.执行管理 API

# # # 4.1 执行路由

```rust
fn execution_routes() -> Router<Hetumind> {
    Router::new()
        .route("/query", get(query_executions))
        .route("/:id", get(get_execution))
        .route("/:id/cancel", post(cancel_execution))
        .route("/:id/retry", post(retry_execution))
        .route("/:id/logs", get(get_execution_logs))
}
```

# # # 4.2 执行 API 实现

```rust
use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
};
use hetumind_core::{
    node::ExecutionData,
    workflow::{Execution, ExecutionId},
};
use modelsql::page::PageResult;
use fusion_web::WebResult;

use crate::domain::workflow::{ExecutionForQuery, ExecutionSvc};

// --- API Data Models ---

/// The standard response for a single execution.
/// It uses the core execution model directly.
pub type ExecutionResponse = Execution;

/// The response for execution logs.
pub type ExecutionLogResponse = Vec<ExecutionData>;

// --- API Handlers ---

/// 查询执行历史
pub async fn query_executions(
    execution_svc: ExecutionSvc,
    Json(input): Json<ExecutionForQuery>,
) -> WebResult<PageResult<ExecutionResponse>> {
    todo!()
}

/// 获取执行详情
pub async fn get_execution(
    execution_svc: ExecutionSvc,
    Path(execution_id): Path<ExecutionId>,
) -> WebResult<ExecutionResponse> {
    todo!()
}

/// 取消执行
pub async fn cancel_execution(
    execution_svc: ExecutionSvc,
    Path(execution_id): Path<ExecutionId>,
) -> WebResult<()> {
    todo!()
}

/// 重试执行
pub async fn retry_execution(
    execution_svc: ExecutionSvc,
    Path(execution_id): Path<ExecutionId>,
) -> WebResult<()> {
    todo!()
}

/// 获取执行日志
pub async fn get_execution_logs(
    execution_svc: ExecutionSvc,
    Path(execution_id): Path<ExecutionId>,
) -> WebResult<ExecutionLogResponse> {
    todo!()
}

# # 5.认证和授权

# # # 5.1 认证服务

认证服务 API 已实现，见：[auth route](../../../ crates/hetumind/hetumind/src/api/auth.rs), [auth domain](../../../ crates/hetumind/hetumind/src/domain/auth/ mod .rs)

# # # 5.2 认证中间件

复用 [WebAuth](../../ crates/ultimates/ultimate-web/src/middleware/web_auth.rs) 中间件，在需要的 * * router* * 上配置此 `Layer`，如：

```rust
let api_router = Router::new()
.nest("/v1", v1_routes().layer(AsyncRequireAuthorizationLayer::new(WebAuth)))
.nest("/auth", auth_routes());
```

以 ` / api/v1` 路径开头的路由应用了 `WebAuth`

# # 6.WebSocket 实时通信

# # # 6.1 WebSocket 路由

```rust
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::Response,
};
use tokio::sync::broadcast;

fn websocket_routes() -> Router<Hetumind> {
    Router::new()
        .route("/executions", get(execution_websocket))
        .route("/workflows/:id", get(workflow_websocket))
}

pub async fn execution_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Hetumind>,
    claims: Claims,
) -> Response {
    ws.on_upgrade(move |socket| handle_execution_socket(socket, state, claims))
}

async fn handle_execution_socket(
    socket: WebSocket,
    state: Hetumind,
    claims: Claims,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut execution_rx = state.execution_events.subscribe();

    // 发送执行状态更新
    let send_task = tokio::spawn(async move {
        while let Ok(event) = execution_rx.recv().await {
            // 过滤用户相关的事件
            if event.user_id == claims.user_id {
                let message = serde_json::to_string(&event).unwrap();
                if sender.send(Message::Text(message)).await.is_err() {
                    break;
                }
            }
        }
    });

    // 处理客户端消息
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        // 处理客户端命令
                        if let Ok(command) = serde_json::from_str::<WebSocketCommand>(&text) {
                            handle_websocket_command(command, &state).await;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }
    });

    // 等待任一任务完成
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
```

# # # 6.2 WebSocket 事件

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub event_type: ExecutionEventType,
    pub execution_id: ExecutionId,
    pub workflow_id: WorkflowId,
    pub user_id: UserId,
    pub timestamp: Timestamp,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEventType {
    ExecutionStarted,
    ExecutionCompleted,
    ExecutionFailed,
    ExecutionCancelled,
    NodeStarted,
    NodeCompleted,
    NodeFailed,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WebSocketCommand {
    SubscribeExecution { execution_id: ExecutionId },
    UnsubscribeExecution { execution_id: ExecutionId },
    SubscribeWorkflow { workflow_id: WorkflowId },
    UnsubscribeWorkflow { workflow_id: WorkflowId },
}
```

# # 7.错误处理

# # # 7.1 API 错误类型

API 错误复用 `WebError`

- [WebError](../../ crates/ultimates/ultimate-web/src/error.rs)

这个 API 设计为 Hetumind 系统提供了完整的 RESTful 接口和实时通信能力，支持现代化的认证授权和错误处理机制。
````
