# hetumind-core

核心库：Workflow 引擎、Task 队列、Expression 表达式、Node Registry。

## Imports

```rust
use hetumind_core::{
    workflow::{
        FlowNode, FlowNodeRef, NodeDescription, NodeExecutionContext,
        ExecutionDataMap, ExecutionData, ExecutionDataItems, NodeConnectionKind,
        WorkflowEngine, ExecutionContext, ExecutionId, ExecutionResult, ExecutionStatus,
        NodeRegistry, NodeType, RegistrationError,
        make_execution_data_map,
    },
    task::{TaskQueue, TaskWorker, QueueTask, TaskResult, WorkerError},
    expression::parse_expression,
    version::Version,
};
```

## FlowNode Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait FlowNode {
    /// 初始化节点
    async fn init(&mut self, _context: &NodeExecutionContext) -> Result<()> {
        Ok(())
    }

    /// 执行节点，返回多输出端口数据
    async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap>;

    /// 获取节点定义
    fn description(&self) -> Arc<NodeDescription>;
}

pub type FlowNodeRef = Arc<dyn FlowNode + Send + Sync>;
```

## WorkflowEngine Trait

```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn execute_workflow(
        &self,
        trigger_data: WorkflowTriggerData,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult, WorkflowExecutionError>;

    async fn pause_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError>;
    async fn resume_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError>;
    async fn cancel_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError>;
    async fn get_execution_status(&self, execution_id: &ExecutionId) -> Result<ExecutionStatus, WorkflowExecutionError>;

    async fn get_execution_metrics(&self, execution_id: &ExecutionId) -> Result<Option<ExecutionMetrics>, WorkflowExecutionError>;
    async fn get_execution_trace(&self, execution_id: &ExecutionId) -> Result<Option<ExecutionTrace>, WorkflowExecutionError>;
}
```

## Execution Types

### ExecutionDataMap

```rust
pub type ExecutionDataMap = HashMap<NodeConnectionKind, Vec<ExecutionDataItems>>;

pub struct ExecutionDataItems {
    pub items: Vec<ExecutionData>,
}

pub struct ExecutionData {
    pub json_value: serde_json::Value,
    pub binary: Option<Vec<u8>>,
}

// 辅助函数
pub fn make_execution_data_map(
    outputs: Vec<(NodeConnectionKind, Vec<ExecutionDataItems>)>
) -> ExecutionDataMap {
    outputs.into_iter().collect()
}
```

### ExecutionStatus

```rust
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed(String),
    Paused { next_task_id: String, reason: String },
    WaitingForInput,
    Cancelled,
}
```

### ExecutionContext

```rust
use hetus::common::ctx::Ctx;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub execution_id: ExecutionId,
    pub workflow: Arc<Workflow>,
    pub ctx: Ctx,
    pub started_at: DateTime<FixedOffset>,
}
```

## NodeExecutionContext

```rust
pub struct NodeExecutionContext {
    pub execution_context: ExecutionContext,
    pub current_node: WorkflowNode,
    pub input_data: ExecutionDataMap,
    pub node_results: HashMap<String, ExecutionDataMap>,
}

impl NodeExecutionContext {
    pub fn current_node(&self) -> Result<&WorkflowNode>;
    pub fn get_input_data(&self, port: &str) -> Result<&ExecutionDataItems>;
    pub fn get_node_result(&self, node_id: &str) -> Option<&ExecutionDataMap>;
    pub fn get_parameter<T: DeserializeOwned>(&self, key: &str) -> Result<T>;
}
```

## TaskQueue Trait

```rust
#[async_trait]
pub trait TaskQueue: Send + Sync {
    async fn initialize(&self) -> Result<(), QueueError>;
    async fn enqueue(&self, task: QueueTask) -> Result<Uuid, QueueError>;
    async fn enqueue_batch(&self, tasks: Vec<QueueTask>) -> Result<Vec<Uuid>, QueueError>;
    async fn dequeue(&self, worker_id: &Uuid, batch_size: usize) -> Result<Vec<(Uuid, QueueTask)>, QueueError>;
    async fn ack(&self, task_id: &Uuid, result: TaskResult) -> Result<(), QueueError>;
    async fn nack(&self, task_id: &Uuid, error: &str, retry: bool) -> Result<(), QueueError>;
    async fn delay(&self, task_id: &Uuid, delay: Duration) -> Result<(), QueueError>;
    async fn cancel(&self, task_id: &Uuid) -> Result<(), QueueError>;
    async fn get_task_status(&self, task_id: &Uuid) -> Result<Option<TaskStatus>, QueueError>;
    async fn get_stats(&self) -> Result<QueueStats, QueueError>;
    async fn cleanup(&self, retention: Duration) -> Result<u64, QueueError>;
}
```

## TaskWorker Trait

```rust
#[async_trait]
pub trait TaskWorker: Send + Sync {
    async fn process_task(&self, task: &QueueTask) -> Result<TaskResult, WorkerError>;
    fn worker_id(&self) -> &Uuid;
    fn batch_size(&self) -> usize { 5 }
    fn should_stop(&self) -> bool;
}
```

## NodeRegistry

```rust
use dashmap::DashMap;

pub struct InnerNodeRegistry {
    nodes: DashMap<NodeType, NodeRef>,
    subnode_providers: DashMap<NodeType, SubNodeRef>,
    llm_suppliers: DashMap<NodeType, LLMSubNodeProviderRef>,
    memory_suppliers: DashMap<NodeType, MemorySubNodeProviderRef>,
    tool_suppliers: DashMap<NodeType, ToolSubNodeProviderRef>,
}

impl InnerNodeRegistry {
    pub fn register_node(&self, executable: NodeRef) -> Result<(), RegistrationError>;
    pub fn get_executor(&self, node_type: &NodeType) -> Option<FlowNodeRef>;
    pub fn get_executor_by_version(&self, node_type: &NodeType, version: &Version) -> Option<FlowNodeRef>;

    pub fn register_llm_supplier(&self, kind: NodeType, provider: LLMSubNodeProviderRef) -> Result<(), RegistrationError>;
    pub fn register_memory_supplier(&self, kind: NodeType, provider: MemorySubNodeProviderRef) -> Result<(), RegistrationError>;
    pub fn register_tool_supplier(&self, kind: NodeType, provider: ToolSubNodeProviderRef) -> Result<(), RegistrationError>;
}
```

## Expression Engine

```rust
use hetumind_core::expression::{parse_expression, Expression};

// 解析表达式
let expr = parse_expression("{{ $json.name }}")?;

// 求值
let context = json!({"name": "John"});
let result = expr.evaluate(&context)?;
// result = "John"
```

## Workflow Definition

```rust
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub nodes: Vec<WorkflowNode>,
    pub connections: Vec<Connection>,
    pub settings: WorkflowSettings,
}

pub struct WorkflowNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub position: Position,
    pub parameters: serde_json::Value,
}

pub struct Connection {
    pub source: String,
    pub source_output: String,
    pub target: String,
    pub target_input: String,
}
```

## Best Practices

1. **节点隔离**: 每个节点应该是无状态的，所有状态通过 Context 传递
2. **错误处理**: 使用 `NodeExecutionError` 明确错误类型
3. **版本管理**: 使用 `Version` 支持节点的多版本
4. **并发安全**: NodeRegistry 使用 DashMap 支持并发访问

## Examples from Codebase

- `hetumind/hetumind-core/src/workflow/flow_node.rs` - FlowNode trait
- `hetumind/hetumind-core/src/workflow/engine.rs` - WorkflowEngine
- `hetumind/hetumind-core/src/task/task_queue.rs` - TaskQueue
