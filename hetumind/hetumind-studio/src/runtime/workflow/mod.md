# 工作流执行引擎

## 概述

本模块实现了基于拓扑排序的工作流执行引擎，支持并发执行和依赖管理。

## 核心组件

### ExecutionGraph

`ExecutionGraph` 是工作流执行图的内部表示，包含：

- **邻接表** (`adjacency`): 存储节点间的依赖关系
- **入度表** (`in_degrees`): 存储每个节点的入度（依赖数量）
- **父节点表** (`parents`): 存储每个节点的父节点列表
- **节点列表** (`nodes`): 所有节点的 ID 列表

### 主要功能

#### 1. 图构建

从 `Workflow` 结构构建执行图：

```rust,ignore
use hetumind_core::workflow::{ExecutionGraph, Workflow, WorkflowSettings, WorkflowMeta, WorkflowStatus, ExecutionMode, ErrorHandlingStrategy};
use hetumind_core::workflow::PinData;
use uuid::Uuid;

let workflow_id = Uuid::now_v7();
let workflow = Workflow {
  id: workflow_id.into(),
  name: "Test Workflow".to_string(),
  status: WorkflowStatus::Draft,
  version: Some(workflow_id.into()),
  settings: WorkflowSettings {
    execution_timeout: None,
    error_handling: Some(ErrorHandlingStrategy::StopOnFirstError),
    execution_mode: Some(ExecutionMode::default()),
    remark: None,
  },
  meta: WorkflowMeta {
    credentials_setup_completed: Some(false),
    template_id: None,
  },
  nodes: Vec::new(),
  connections: HashMap::default(),
  pin_data: PinData::default(),
  static_data: None,
};
let graph = ExecutionGraph::new(&workflow);
```

#### 2. 拓扑排序执行

基于 Kahn 算法的拓扑排序，支持：

- 循环依赖检测
- 并发执行就绪节点
- 动态依赖更新

#### 3. 数据传递

- 汇集父节点输出作为当前节点输入
- 支持多个父节点的数据合并
- 保持数据血缘关系

## 执行流程

1. **构建执行图**: 从工作流定义构建内部图表示
2. **循环依赖检测**: 使用 DFS 检测图中是否存在循环
3. **初始化状态**: 设置执行结果存储和依赖计数
4. **找到起始节点**: 识别入度为 0 的节点作为执行起点
5. **并发执行循环**:
   - 并发执行所有就绪节点
   - 收集执行结果
   - 更新子节点依赖计数
   - 将新就绪的节点加入队列
6. **结果汇总**: 组装最终执行结果

## 错误处理

支持三种错误处理策略：

- `StopOnFirstError`: 遇到第一个错误时停止整个工作流
- `ContinueOnError`: 继续执行其他节点，但记录错误
- `ErrorNode`: 使用专门的错误处理节点

## 并发特性

- **节点级并发**: 同一层级的独立节点可以并发执行
- **批次执行**: 按拓扑层级分批执行节点
- **资源控制**: 通过 `ConcurrencyController` 控制并发度

## 测试

包含完整的单元测试：

- 图构建测试
- 循环依赖检测测试
- 依赖关系验证测试

运行测试：

```bash
cargo test -p hetumind-studio test_execution_graph
```

## 使用示例

```rust,ignore
use std::sync::Arc;

use fusion_common::ctx::Ctx;
use hetumind_studio::runtime::workflow::WorkflowEngineImpl;
use hetumind_studio::runtime::execution::ExecutionStore;
use hetumind_studio::runtime::checkpoint::{CheckpointError, ExecutionCheckpoint};
use hetumind_core::workflow::NodeRegistry;
use hetumind_core::workflow::{
    WorkflowEngine, ExecutionContext, Workflow, WorkflowSettings, WorkflowMeta,
    WorkflowStatus, ExecutionMode, ErrorHandlingStrategy, ExecutionId, ExecutionStatus,
    RetryConfig, WorkflowExecutionError, Execution
};
use hetumind_core::workflow::ExecutionData;
use hetumind_core::workflow::PinData;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Clone)]
struct MockExecutionStore;

#[async_trait]
impl ExecutionStore for MockExecutionStore {
    async fn save_execution(&self, _execution: &Execution) -> Result<(), WorkflowExecutionError> {
        todo!()
    }
    async fn get_execution(&self, _id: ExecutionId) -> Result<Option<Execution>, WorkflowExecutionError> {
        todo!()
    }
    async fn get_execution_status(&self, _execution_id: ExecutionId) -> Result<ExecutionStatus, WorkflowExecutionError> {
        todo!()
    }
    async fn update_execution_status(&self, _id: ExecutionId, _status: ExecutionStatus) -> Result<(), WorkflowExecutionError> {
        todo!()
    }
    async fn save_checkpoint(&self, _checkpoint: ExecutionCheckpoint) -> Result<(), CheckpointError> {
        todo!()
    }
    async fn load_latest_checkpoint(&self, _execution_id: ExecutionId) -> Result<Option<ExecutionCheckpoint>, CheckpointError> {
        todo!()
    }
}

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个测试工作流
    let workflow_id = Uuid::now_v7();
    let test_workflow = Workflow {
        id: workflow_id.into(),
        name: "Test Workflow".to_string(),
        status: WorkflowStatus::Draft,
        version: Some(workflow_id.into()),
        settings: WorkflowSettings {
            execution_timeout: None,
            error_handling: Some(ErrorHandlingStrategy::StopOnFirstError),
            execution_mode: Some(ExecutionMode::default()),
            remark: None,
        },
        meta: WorkflowMeta {
            credentials_setup_completed: Some(false),
            template_id: None,
        },
        nodes: Vec::new(),
        connections: HashMap::default(),
        pin_data: PinData::default(),
        static_data: None,
    };

    // 创建执行引擎
    let node_registry = Arc::new(NodeRegistry::new());
    let execution_store = Arc::new(MockExecutionStore);
    let engine = WorkflowEngineImpl::new(
        node_registry,
        execution_store,
    );

    // 执行工作流
    let trigger_data: Option<Vec<ExecutionData>> = None;
    let context = ExecutionContext::new(Arc::new(test_workflow), Ctx::new_super_admin());

    // The real implementation is not complete, so this will panic.
    // let result = engine.execute_workflow(
    //     &test_workflow,
    //     trigger_data,
    //     &context,
    // ).await;

    # Ok(())
    #
}
```

## 性能特点

- **O(V + E)** 时间复杂度的拓扑排序
- **最小化内存拷贝**: 使用引用传递数据
- **高效的依赖更新**: 使用哈希表快速查找
- **并发友好**: 支持多线程安全的并发执行
