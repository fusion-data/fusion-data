# Hetumind 执行引擎设计

## 1. 执行引擎概述

Hetumind 执行引擎是系统的核心组件，负责工作流的调度、执行和管理。它采用基于 Rust 的 async/await 模式，提供高性能、类型安全的工作流执行能力。

### 1.1 设计目标

- **高性能**: 利用 Rust 零成本抽象和 Tokio 异步运行时
- **并发安全**: 利用 Rust 的所有权系统保证内存安全
- **可扩展**: 支持水平扩展和负载均衡
- **容错性**: 具备故障恢复和重试机制
- **可观测**: 内置监控和日志记录

### 1.2 核心组件

```mermaid
flowchart TB
    subgraph "执行引擎核心"
        Engine[WorkflowEngine]
        Scheduler[TaskScheduler]
        Executor[NodeExecutor]
        Monitor[ExecutionMonitor]
    end

    subgraph "执行状态管理"
        State[ExecutionState]
        Queue[TaskQueue]
        Registry[NodeRegistry]
    end

    subgraph "并发控制"
        Pool[ThreadPool]
        Semaphore[Semaphore]
        RateLimit[RateLimiter]
    end

    subgraph "持久化"
        DB[(Database)]
        Cache[(Redis)]
        Storage[(FileStorage)]
    end

    Engine --> Scheduler
    Engine --> Executor
    Engine --> Monitor

    Scheduler --> State
    Scheduler --> Queue
    Executor --> Registry

    Pool --> Executor
    Semaphore --> Scheduler
    RateLimit --> Engine

    State --> DB
    Queue --> Cache
    Monitor --> Storage

    style Engine fill:#007acc,color:#fff
    style Scheduler fill:#4CAF50,color:#fff
    style Executor fill:#FF9800,color:#fff
```

## 2. 核心执行结构

### 2.1 工作流执行引擎

- [ExecutionConfig](../../../crates/hetumind/hetumind-core/src/workflow/config.rs)
- [WorkflowEngineImpl](../../../crates/hetumind/hetumind-studio/src/runtime/workflow/engine.rs)

### 2.2 任务调度器

- [TaskScheduler](../../../crates/hetumind/hetumind-studio/src/runtime/task/task_scheduler.rs)

### 2.3 节点执行器

- [NodeExecutorImpl](../../../crates/hetumind/hetumind-studio/src/runtime/node/node_executor.rs)

## 3. 执行流程设计

### 3.1 工作流执行状态机

```mermaid
stateDiagram-v2
    [*] --> Created: 创建执行
    Created --> Queued: 加入队列
    Queued --> Running: 开始执行

    Running --> Paused: 暂停请求
    Paused --> Running: 恢复执行

    Running --> Success: 所有节点成功
    Running --> Failed: 节点执行失败
    Running --> Cancelled: 取消请求
    Running --> Timeout: 执行超时

    Success --> [*]
    Failed --> [*]
    Cancelled --> [*]
    Timeout --> [*]

    note right of Running
        执行状态包括:
        - 节点调度
        - 依赖检查
        - 并发控制
        - 错误处理
    end note

    note right of Paused
        暂停状态保持:
        - 当前执行状态
        - 待执行任务队列
        - 部分执行结果
    end note
```

### 3.2 节点执行流程

```mermaid
sequenceDiagram
    participant Engine as WorkflowEngine
    participant Scheduler as TaskScheduler
    participant Executor as NodeExecutor
    participant Node as NodeImplementation
    participant Store as ExecutionStore

    Engine->>Scheduler: 提交工作流执行
    Scheduler->>Scheduler: 分析依赖关系
    Scheduler->>Scheduler: 创建初始任务队列

    loop 执行循环
        Scheduler->>Scheduler: 检查就绪任务
        Scheduler->>Executor: 执行节点任务

        par 并发执行节点
            Executor->>Node: 执行节点逻辑
            Node->>Node: 处理输入数据
            Node->>Node: 执行业务逻辑
            Node-->>Executor: 返回输出数据
        end

        Executor-->>Scheduler: 节点执行结果
        Scheduler->>Store: 保存执行结果
        Scheduler->>Scheduler: 更新依赖状态
        Scheduler->>Scheduler: 调度下游任务
    end

    Scheduler-->>Engine: 工作流执行完成
    Engine->>Store: 更新最终状态
```

### 3.3 错误处理流程

```mermaid
flowchart TD
    A[节点执行失败] --> B{检查重试配置}

    B -->|有重试| C[延迟重试]
    B -->|无重试| D{检查错误处理策略}

    C --> E[重试次数检查]
    E -->|未达上限| F[重新执行节点]
    E -->|达到上限| D

    D -->|停止工作流| G[标记执行失败]
    D -->|继续执行| H[跳过当前节点]
    D -->|错误输出| I[输出错误数据]

    F --> J{执行结果}
    J -->|成功| K[继续工作流]
    J -->|失败| E

    H --> L[调度下游节点]
    I --> L

    G --> M[清理资源]
    K --> N[正常流程继续]
    L --> N

    M --> O[结束]
    N --> O

    style A fill:#f44336,color:#fff
    style G fill:#f44336,color:#fff
    style K fill:#4CAF50,color:#fff
    style O fill:#2196F3,color:#fff
```

## 4. 并发控制机制

### 4.1 并发控制器

- [ConcurrencyController](../../../crates/hetumind/hetumind-studio/src/runtime/task/concurrency_controller.rs)

### 4.2 资源监控

- [ResourceMonitor](../../../crates/hetumind/hetumind-studio/src/runtime/task/resource_monitor.rs)

## 5. 执行监控和观测

### 5.1 执行监控器

- [ExecutionMonitor](../../../crates/hetumind/hetumind-studio/src/runtime/task/execution_monitor.rs)

### 5.2 性能指标定义

- [ExecutionMetrics](../../../crates/hetumind/hetumind-core/src/metrics/mod.rs)
- [ResourceUsage](../../../crates/hetumind/hetumind-core/src/metrics/mod.rs)

## 6. 容错和恢复机制

### 6.1 故障检测和恢复

```mermaid
flowchart TD
    A[检测到故障] --> B{故障类型分析}

    B -->|节点执行失败| C[节点级别恢复]
    B -->|网络故障| D[重试机制]
    B -->|系统资源不足| E[资源清理]
    B -->|外部服务不可用| F[服务降级]

    C --> G[检查重试配置]
    G -->|允许重试| H[延迟重试]
    G -->|不允许重试| I[标记节点失败]

    D --> J[指数退避重试]
    J --> K{重试成功?}
    K -->|是| L[继续执行]
    K -->|否| M[标记连接失败]

    E --> N[清理内存]
    N --> O[释放资源]
    O --> P[暂停新任务]

    F --> Q[使用备用服务]
    Q --> R{备用可用?}
    R -->|是| L
    R -->|否| S[降级处理]

    H --> T{执行结果}
    T -->|成功| L
    T -->|失败| U[递增重试计数]
    U --> G

    I --> V[错误处理策略]
    M --> V
    S --> V

    V -->|停止工作流| W[终止执行]
    V -->|继续执行| X[跳过失败节点]
    V -->|错误输出| Y[输出错误数据]

    L --> Z[正常执行流程]
    W --> AA[清理资源并退出]
    X --> Z
    Y --> Z

    style A fill:#f44336,color:#fff
    style W fill:#f44336,color:#fff
    style L fill:#4CAF50,color:#fff
    style Z fill:#4CAF50,color:#fff
```

### 6.2 检查点和状态恢复

- [CheckpointManager](../../../crates/hetumind/hetumind-studio/src/runtime/checkpoint/checkpoint_manager.rs)

这个执行引擎设计提供了完整的工作流执行能力，包括高性能的任务调度、并发控制、错误处理和监控能力，为 Hetumind 系统提供了可靠的执行基础。
