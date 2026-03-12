---
name: hetumind
description: Hetumind AI Agent/Flow platform, workflow engine, FlowNode, NodeRegistry, TaskQueue, LLM node, trigger node, expression engine, AI 工作流平台
globs:
  - "hetumind/**/*.rs"
---

# Hetumind - AI Agent/Flow 平台

## Quick Reference

| Module | Import | Key Types |
|--------|--------|-----------|
| Core | `use hetumind_core::*;` | `FlowNode`, `WorkflowEngine`, `TaskQueue`, `NodeRegistry` |
| Nodes | `use hetumind_nodes::*;` | `ManualTriggerNode`, `OpenaiModelNode`, `IfNode` |
| Context | `use hetumind_context::*;` | `NodeExecutionContext`, `ExecutionContext` |
| Studio | `use hetumind_studio::*;` | `WorkflowSvc`, `CredentialSvc` |

## Core Traits

### FlowNode
```rust
#[async_trait]
pub trait FlowNode {
    async fn init(&mut self, context: &NodeExecutionContext) -> Result<()>;
    async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap>;
    fn description(&self) -> Arc<NodeDescription>;
}
```

### WorkflowEngine
```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn execute_workflow(&self, trigger: WorkflowTriggerData, ctx: &ExecutionContext) -> Result<ExecutionResult>;
    async fn pause_execution(&self, id: &ExecutionId) -> Result<()>;
    async fn resume_execution(&self, id: &ExecutionId) -> Result<()>;
    async fn cancel_execution(&self, id: &ExecutionId) -> Result<()>;
}
```

### TaskQueue
```rust
#[async_trait]
pub trait TaskQueue: Send + Sync {
    async fn enqueue(&self, task: QueueTask) -> Result<Uuid>;
    async fn dequeue(&self, worker_id: &Uuid, batch_size: usize) -> Result<Vec<(Uuid, QueueTask)>>;
    async fn ack(&self, task_id: &Uuid, result: TaskResult) -> Result<()>;
    async fn nack(&self, task_id: &Uuid, error: &str, retry: bool) -> Result<()>;
}
```

## Core Patterns

### 自定义节点
```rust
use hetumind_core::workflow::{FlowNode, NodeExecutionContext, ExecutionDataMap};
use async_trait::async_trait;

pub struct MyCustomNode {
    definition: Arc<NodeDescription>,
}

#[async_trait]
impl FlowNode for MyCustomNode {
    fn description(&self) -> Arc<NodeDescription> {
        Arc::clone(&self.definition)
    }

    async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap> {
        // 从输入端口获取数据
        let input = context.get_input_data("main")?;

        // 处理数据
        let output = process(input)?;

        // 返回输出
        Ok(make_execution_data_map(vec![
            (NodeConnectionKind::Main, vec![ExecutionDataItems::new_item(output)])
        ]))
    }
}
```

### 节点注册
```rust
use hetumind_core::workflow::NodeRegistry;

let registry = NodeRegistry::new();
let node = Arc::new(MyCustomNode::new()?);
registry.register_node(node)?;
```

### LLM Provider 注册
```rust
use hetumind_core::workflow::SubNodeProvider;

let llm_provider = Arc::new(OpenAILLMProvider::new(config));
registry.register_llm_supplier(NodeType::OpenAI, llm_provider)?;
```

## Node Types

| Category | Nodes |
|----------|-------|
| **Trigger** | ManualTrigger, ScheduleTrigger, WebhookTrigger, EmailTrigger |
| **Core** | IfNode, NoOpNode, LoopOverItems |
| **LM** | OpenAI, DeepSeek, Moonshot |
| **Store** | SimpleMemoryNode |
| **Integration** | HttpRequest |

## References (按需加载)

- [core](references/core.md) - workflow, engine, task queue, expression
- [nodes](references/nodes.md) - trigger, core, lm, store nodes
- [context](references/context.md) - execution context, services
- [studio](references/studio.md) - runtime, endpoint, API

## Related Skills
- `hetu`: 核心库模式，特别是 hetu-ai
- `cluster-node`: NodeRegistry, SubNodeProvider
- `sql-database`: BMC/Service 层
