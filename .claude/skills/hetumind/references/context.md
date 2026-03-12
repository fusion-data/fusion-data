# hetumind-context

上下文管理：NodeExecutionContext、ExecutionContext、服务访问。

## Imports

```rust
use hetumind_context::{
    NodeExecutionContext, ExecutionContext,
    services::{CredentialService, WorkflowService},
};
```

## ExecutionContext

```rust
use hetus::common::ctx::Ctx;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub execution_id: ExecutionId,
    pub workflow: Arc<Workflow>,
    pub ctx: Ctx,
    pub started_at: DateTime<FixedOffset>,
}

impl ExecutionContext {
    pub fn new(workflow: Arc<Workflow>, ctx: Ctx) -> Self {
        Self {
            execution_id: ExecutionId::new_v4(),
            workflow,
            ctx,
            started_at: now_offset(),
        }
    }

    pub fn user_id(&self) -> i64 {
        self.ctx.user_id()
    }

    pub fn tenant_id(&self) -> i64 {
        self.ctx.tenant_id()
    }
}
```

## NodeExecutionContext

```rust
use hetumind_core::workflow::{ExecutionDataMap, NodeRegistry};
use std::collections::HashMap;

pub struct NodeExecutionContext {
    pub execution_context: ExecutionContext,
    pub current_node: WorkflowNode,
    pub input_data: ExecutionDataMap,
    pub node_results: HashMap<String, ExecutionDataMap>,
    pub registry: Arc<NodeRegistry>,
}

impl NodeExecutionContext {
    /// 获取当前节点
    pub fn current_node(&self) -> Result<&WorkflowNode> {
        Ok(&self.current_node)
    }

    /// 获取输入数据
    pub fn get_input_data(&self, port: &str) -> Result<&ExecutionDataItems> {
        self.input_data.get(&NodeConnectionKind::from(port))
            .and_then(|items| items.first())
            .ok_or_else(|| NodeExecutionError::NoInputData(port.into()))
    }

    /// 获取参数
    pub fn get_parameter<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let value = &self.current_node.parameters[key];
        serde_json::from_value(value.clone())
            .map_err(|e| NodeExecutionError::ParameterParse(key.into(), e.to_string()))
    }

    /// 获取前一个节点的输出
    pub fn get_previous_output(&self, node_name: &str) -> Option<&ExecutionDataMap> {
        self.node_results.get(node_name)
    }

    /// 获取 NodeRegistry
    pub fn get_registry(&self) -> &Arc<NodeRegistry> {
        &self.registry
    }

    /// 获取聊天消息历史
    pub fn get_chat_messages(&self) -> Result<Vec<ChatMessage>> {
        // 从输入数据构建消息
        let input = self.get_input_data("main")?;
        let mut messages = Vec::new();

        for item in &input.items {
            if let Some(msgs) = item.json_value.get("messages") {
                messages.extend(serde_json::from_value::<Vec<ChatMessage>>(msgs.clone())?);
            }
        }

        Ok(messages)
    }

    /// 获取用户上下文
    pub fn user_context(&self) -> &Ctx {
        &self.execution_context.ctx
    }
}
```

## Services

### CredentialService

```rust
use hetusql::ModelManager;

pub struct CredentialService {
    mm: ModelManager,
}

impl CredentialService {
    pub async fn get_credential(&self, credential_id: &str) -> Result<Credential> {
        let cred = CredentialBmc::find_by_id(&self.mm, credential_id).await?
            .ok_or_else(|| Error::CredentialNotFound(credential_id.into()))?;

        // 解密敏感数据
        let decrypted = decrypt_credential(&cred.encrypted_value)?;

        Ok(Credential {
            id: cred.id,
            name: cred.name,
            credential_type: cred.credential_type,
            value: decrypted,
        })
    }

    pub async fn list_credentials(&self, filter: CredentialFilter) -> Result<Vec<CredentialEntity>> {
        CredentialBmc::list(&self.mm, Some(filter), None).await
    }
}
```

### WorkflowService

```rust
pub struct WorkflowService {
    mm: ModelManager,
}

impl WorkflowService {
    pub async fn get_workflow(&self, workflow_id: &str) -> Result<Workflow> {
        let entity = WorkflowBmc::find_by_id(&self.mm, workflow_id).await?
            .ok_or_else(|| Error::WorkflowNotFound(workflow_id.into()))?;

        // 解析 workflow 定义
        let workflow: Workflow = serde_json::from_value(entity.definition)?;

        Ok(workflow)
    }

    pub async fn save_execution(&self, execution: &ExecutionResult) -> Result<()> {
        let entity = ExecutionEntity {
            id: execution.id.to_string(),
            workflow_id: execution.workflow_id.clone(),
            status: execution.status.clone(),
            started_at: execution.started_at,
            completed_at: execution.completed_at,
            result: serde_json::to_value(&execution.result)?,
        };

        ExecutionBmc::upsert(&self.mm, entity).await
    }
}
```

## Context in Node Execution

```rust
impl NodeExecutionContext {
    /// 表达式求值
    pub fn evaluate_expression(&self, expr: &str) -> Result<Value> {
        let parsed = parse_expression(expr)?;

        // 构建上下文变量
        let context = json!({
            "$json": self.get_input_data("main")?.first()?.json_value,
            "$execution": {
                "id": self.execution_context.execution_id.to_string(),
                "started_at": self.execution_context.started_at,
            },
            "$workflow": {
                "id": self.execution_context.workflow.id,
                "name": self.execution_context.workflow.name,
            },
        });

        parsed.evaluate(&context)
    }
}
```

## Best Practices

1. **上下文传递**: 始终通过 ExecutionContext 传递用户信息
2. **凭证管理**: 使用 CredentialService 获取和解密凭证
3. **表达式求值**: 使用 evaluate_expression 处理动态参数
4. **错误处理**: 使用明确的错误类型

## Examples from Codebase

- `hetumind/hetumind-context/src/lib.rs` - Context 定义
- `hetumind/hetumind-studio/src/runtime/executor.rs` - 执行器使用 context
