# hetumind-studio

Web 服务：Runtime、Endpoint、API。

## Imports

```rust
use hetumind_studio::{
    runtime::{WorkflowExecutor, ExecutionManager},
    endpoint::api::v1::*,
    domain::{WorkflowSvc, CredentialSvc, ExecutionSvc},
};
```

## Runtime

### WorkflowExecutor

```rust
use hetumind_core::workflow::{WorkflowEngine, ExecutionResult};
use hetumind_context::ExecutionContext;

pub struct WorkflowExecutor {
    registry: Arc<NodeRegistry>,
    storage: Arc<dyn SessionStorage>,
    mm: ModelManager,
}

impl WorkflowExecutor {
    pub async fn execute(&self, workflow_id: &str, ctx: Ctx) -> Result<ExecutionResult> {
        // 加载 workflow 定义
        let workflow = self.load_workflow(workflow_id).await?;

        // 创建执行上下文
        let exec_ctx = ExecutionContext::new(Arc::new(workflow), ctx);

        // 找到触发器节点
        let trigger_node = self.find_trigger_node(&exec_ctx.workflow)?;

        // 从触发器开始执行
        let result = self.execute_from(trigger_node, &exec_ctx).await?;

        // 保存执行结果
        self.save_execution(&result).await?;

        Ok(result)
    }

    async fn execute_from(&self, start_node: &WorkflowNode, ctx: &ExecutionContext) -> Result<ExecutionResult> {
        let mut current = start_node;
        let mut results = HashMap::new();

        loop {
            // 获取节点执行器
            let executor = self.registry.get_executor(&current.node_type)
                .ok_or_else(|| Error::NodeNotFound(current.node_type.clone()))?;

            // 构建节点上下文
            let node_ctx = NodeExecutionContext {
                execution_context: ctx.clone(),
                current_node: current.clone(),
                input_data: self.collect_input_data(current, &results),
                node_results: results.clone(),
                registry: Arc::clone(&self.registry),
            };

            // 执行节点
            let output = executor.execute(&node_ctx).await?;
            results.insert(current.id.clone(), output.clone());

            // 找到下一个节点
            match self.find_next_node(current, &output, &ctx.workflow) {
                Some(next) => current = next,
                None => break,
            }
        }

        Ok(ExecutionResult {
            execution_id: ctx.execution_id,
            workflow_id: ctx.workflow.id.clone(),
            status: ExecutionStatus::Completed,
            started_at: ctx.started_at,
            completed_at: Some(now_offset()),
            result: results,
        })
    }
}
```

### ExecutionManager

```rust
use hetumind_core::task::{TaskQueue, TaskWorker, QueueTask};

pub struct ExecutionManager {
    queue: Arc<dyn TaskQueue>,
    executor: Arc<WorkflowExecutor>,
}

impl ExecutionManager {
    pub async fn submit(&self, workflow_id: &str, ctx: Ctx) -> Result<Uuid> {
        let task = QueueTask::WorkflowExecution {
            workflow_id: workflow_id.to_string(),
            ctx: ctx.clone(),
        };

        let task_id = self.queue.enqueue(task).await?;
        Ok(task_id)
    }

    pub async fn get_status(&self, execution_id: &Uuid) -> Result<ExecutionStatus> {
        self.queue.get_task_status(execution_id).await?
            .ok_or_else(|| Error::ExecutionNotFound(execution_id.to_string()))
    }

    pub async fn cancel(&self, execution_id: &Uuid) -> Result<()> {
        self.queue.cancel(execution_id).await
    }
}
```

## Domain Services

### WorkflowSvc

```rust
use hetusql::ModelManager;
use hetus::core::DataError;

pub struct WorkflowSvc {
    mm: ModelManager,
}

impl WorkflowSvc {
    pub async fn list(&self, filter: Option<WorkflowFilter>) -> Result<Vec<WorkflowEntity>> {
        WorkflowBmc::list(&self.mm, filter, None).await
    }

    pub async fn get(&self, id: &str) -> Result<WorkflowEntity> {
        WorkflowBmc::find_by_id(&self.mm, id).await?
            .ok_or_else(|| DataError::not_found("Workflow not found"))
    }

    pub async fn create(&self, workflow: WorkflowForCreate) -> Result<String> {
        let id = workflow.id.clone().unwrap_or_else(|| Uuid::now_v7().to_string());
        WorkflowBmc::insert(&self.mm, workflow).await?;
        Ok(id)
    }

    pub async fn update(&self, id: &str, workflow: WorkflowForUpdate) -> Result<()> {
        WorkflowBmc::update_by_id(&self.mm, id, workflow).await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        WorkflowBmc::delete_by_id(&self.mm, id).await
    }
}
```

### CredentialSvc

```rust
pub struct CredentialSvc {
    mm: ModelManager,
    encryption_key: String,
}

impl CredentialSvc {
    pub async fn create(&self, cred: CredentialForCreate) -> Result<String> {
        // 加密敏感数据
        let encrypted = encrypt(&cred.value, &self.encryption_key)?;

        let entity = CredentialEntity {
            id: Uuid::now_v7().to_string(),
            name: cred.name,
            credential_type: cred.credential_type,
            encrypted_value: encrypted,
            tenant_id: cred.tenant_id,
            created_at: now_offset(),
        };

        CredentialBmc::insert(&self.mm, entity).await
    }

    pub async fn decrypt(&self, id: &str) -> Result<Value> {
        let cred = CredentialBmc::find_by_id(&self.mm, id).await?
            .ok_or_else(|| DataError::not_found("Credential not found"))?;

        decrypt(&cred.encrypted_value, &self.encryption_key)
    }
}
```

## API Endpoints

### Workflow API

```rust
use utoipa_axum::router::OpenApiRouter;

pub fn routes() -> OpenApiRouter<Application> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(list_workflows))
        .routes(utoipa_axum::routes!(get_workflow))
        .routes(utoipa_axum::routes!(create_workflow))
        .routes(utoipa_axum::routes!(update_workflow))
        .routes(utoipa_axum::routes!(delete_workflow))
        .routes(utoipa_axum::routes!(execute_workflow))
        .routes(utoipa_axum::routes!(get_execution_status))
}

#[utoipa::path(post, path = "/item/{id}/execute", tag = "Workflows")]
async fn execute_workflow(
    Path(id): Path<String>,
    State(executor): State<Arc<WorkflowExecutor>>,
    ctx: Ctx,
) -> WebResult<IdUuidResult> {
    let execution_id = executor.execute(&id, ctx).await?;
    ok_json!(IdUuidResult::from(execution_id.as_uuid()))
}

#[utoipa::path(get, path = "/execution/{id}/status", tag = "Workflows")]
async fn get_execution_status(
    Path(id): Path<Uuid>,
    State(manager): State<Arc<ExecutionManager>>,
) -> WebResult<ExecutionStatusResult> {
    let status = manager.get_status(&id).await?;
    ok_json!(ExecutionStatusResult::from(status))
}
```

### Credential API

```rust
pub fn routes() -> OpenApiRouter<Application> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(list_credentials))
        .routes(utoipa_axum::routes!(create_credential))
        .routes(utoipa_axum::routes!(delete_credential))
}

#[utoipa::path(post, path = "/item", tag = "Credentials")]
async fn create_credential(
    cred_svc: CredentialSvc,
    Json(input): Json<CredentialForCreate>,
) -> WebResult<IdStringResult> {
    let id = cred_svc.create(input).await?;
    ok_json!(IdStringResult::from(id))
}
```

## Web Server Setup

```rust
use hetus::web::WebServerBuilder;
use tower_http::{trace::TraceLayer, cors::{CorsLayer, cors}};
use jieyuan_core::web::path_authz_middleware;

pub async fn init_web(app: Application) -> Result<(), DataError> {
    let router = Router::new()
        .nest("/api", api::routes())
        .with_state(app.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_methods(cors::Any).allow_origin(cors::Any))
        .layer(from_fn_with_state(app, path_authz_middleware));  // Jieyuan 认证

    WebServerBuilder::new(router)
        .with_shutdown(app.shutdown_recv().await)
        .build()
        .await?;

    Ok(())
}
```

## Best Practices

1. **异步执行**: 使用 TaskQueue 异步执行 workflow
2. **凭证加密**: 敏感凭证必须加密存储
3. **认证集成**: 使用 Jieyuan 中间件进行认证
4. **执行追踪**: 记录执行日志用于调试

## Examples from Codebase

- `hetumind/hetumind-studio/src/runtime/executor.rs` - Workflow 执行器
- `hetumind/hetumind-studio/src/endpoint/api/v1/workflows.rs` - Workflow API
- `hetumind/hetumind-studio/src/main.rs` - 服务启动
