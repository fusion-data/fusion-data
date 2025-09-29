# hetuflow 架构设计文档

## 概述

hetuflow 是一个现代化、高性能的分布式任务调度系统。该系统通过 **WebSocket** 全双工通信、**PostgreSQL** 强一致性存储、**Rust** 类型安全保障，实现了高性能、高可靠性的任务调度能力。系统由两个核心二进制程序构成：**hetuflow-server**（核心协调节点）和 **hetuflow-agent**（任务执行单元）。

### 核心特性

- **WebSocket 全双工通信**：支持 Server 主动推送任务和 Agent 主动上报状态
- **类型安全保障**：全程类型安全的数据库操作和通信协议
- **强一致性存储**：基于 PostgreSQL 事务保证数据一致性
- **现代化架构**：Application 容器模式 + ModelManager + BMC 分层设计
- **网络穿透友好**：基于 HTTP/HTTPS，易于穿透防火墙和代理

## 系统架构概览

### 核心组件

- **hetuflow-core**: [`hetuflow-core/src/`](../../../hetuflow-core/src/) - 共享核心库，定义通信协议和数据模型
- **hetuflow-server**: [`hetuflow-server/src/`](../../../hetuflow-server/src/) - 核心协调节点，负责任务编排和分发
- **hetuflow-agent**: [`hetuflow-agent/src/`](../../../hetuflow-agent/src/) - 任务执行单元，负责具体任务执行

### 核心技术栈

- **编程语言**: Rust 2024 Edition
- **数据库**: PostgreSQL + pgvector 扩展
- **ORM**: modelsql (基于 sea-query + sqlx)
- **通信协议**: WebSocket (全双工)
- **异步运行时**: Tokio
- **序列化**: Serde JSON
- **Web 框架**: Axum
- **认证**: JWE (JSON Web Encryption)

## 详细组件设计

### 1. hetuflow-server（核心协调节点）

**核心职责**:
- 任务调度和分发管理
- Agent 连接和状态管理
- Web API 和管理界面
- 数据持久化和事务处理

**主要模块**:
- `application/`: 应用容器和依赖管理
- `scheduler/`: 任务调度引擎
- `broker/`: 任务分发和负载均衡
- `endpoint/`: HTTP API 端点
- `service/`: 业务逻辑服务层
- `infra/`: 基础设施层（BMC 数据库操作）

### 2. hetuflow-agent（任务执行单元）

**核心职责**:
- 接收并执行 Server 下发的任务
- 任务状态监控和上报
- 资源管理和进程控制
- 自动重连和故障恢复

**主要模块**:
- `application/`: 应用容器
- `connection/`: WebSocket 连接管理
- `executor/`: 任务执行器
- `process/`: 进程管理和资源控制
- `scheduler/`: 任务调度和命令处理

## 系统架构设计

### 分层架构设计

hetuflow 采用现代化的分层架构，遵循单一职责、依赖倒置和开闭原则：

```mermaid
graph TB
    subgraph "应用层 (Application Layer)"
        subgraph "hetuflow-server"
            S_APP[ServerApplication]
            S_SCHED[SchedulerSvc]
            S_BROKER[BrokerSvc]
            S_API[HTTP API]
            S_WS[WebSocket Server]
        end

        subgraph "hetuflow-agent"
            A_APP[AgentApplication]
            A_EXEC[TaskExecutor]
            A_SCHED[TaskScheduler]
            A_CONN[ConnectionManager]
        end
    end

    subgraph "基础设施层 (Infrastructure Layer)"
        subgraph "fusion-core"
            FC_APP[Application Container]
            FC_DB[Database Manager]
            FC_ERROR[Error Handling]
        end

        subgraph "modelsql"
            MS_MM[ModelManager]
            MS_BMC[DbBmc Layer]
            MS_QUERY[Query Builder]
        end
    end

    subgraph "核心层 (Core Layer)"
        subgraph "hetuflow-core"
            HC_PROTO[Protocol Definitions]
            HC_MODELS[Data Models]
            HC_TYPES[Core Types]
        end
    end

    subgraph "存储层 (Storage Layer)"
        PG[(PostgreSQL)]
    end

    %% Dependencies
    S_APP --> FC_APP
    S_SCHED --> MS_MM
    S_BROKER --> HC_PROTO
    S_API --> FC_ERROR

    A_APP --> FC_APP
    A_EXEC --> MS_BMC
    A_CONN --> HC_PROTO

    MS_MM --> PG
    MS_BMC --> PG
    MS_QUERY --> PG

    HC_PROTO --> HC_TYPES
    HC_MODELS --> HC_TYPES

    style S_APP fill:#e3f2fd
    style A_APP fill:#e8f5e8
    style FC_APP fill:#fff3e0
    style MS_MM fill:#f3e5f5
    style HC_PROTO fill:#e1f5fe
    style PG fill:#ffcda8
```

### 模块职责定义

**应用层 (Application Layer)**:
- **hetuflow-server**: 负责任务调度、Agent管理、API服务
- **hetuflow-agent**: 负责任务执行、状态上报、资源管理

**基础设施层 (Infrastructure Layer)**:
- **fusion-core**: 提供 Application 容器、错误处理、配置管理等基础功能
- **modelsql**: 提供 ModelManager、DbBmc、Query Builder 等数据库抽象层

**核心层 (Core Layer)**:
- **hetuflow-core**: 定义通信协议、数据模型、类型规范的共享核心库

**存储层 (Storage Layer)**:
- **PostgreSQL**: 提供 ACID 事务保证的持久化存储

### 组件关系与交互

```mermaid
graph TB
    subgraph "用户交互层"
        USER[用户/管理员]
        WEB[Web 界面]
        API[REST API]
    end

    subgraph "hetuflow-server 核心服务"
        subgraph "应用容器"
            SRV_APP[ServerApplication]
        end

        subgraph "调度服务"
            SCHEDULER[SchedulerSvc]
            TASK_GEN[TaskGenerationSvc]
            DISPATCHER[TaskDispatcher]
        end

        subgraph "代理服务"
            BROKER[BrokerSvc]
            LOAD_BAL[LoadBalancer]
        end

        subgraph "API 网关"
            API_GATEWAY[API Gateway]
            WS_SERVER[WebSocket Server]
            AUTH[Auth Service]
        end

        subgraph "数据访问层"
            JOB_BMC[JobBmc]
            TASK_BMC[TaskBmc]
            AGENT_BMC[AgentBmc]
            SCHED_BMC[ScheduleBmc]
        end
    end

    subgraph "hetuflow-agent 集群"
        subgraph "Agent 1"
            A1_APP[AgentApplication]
            A1_EXEC[TaskExecutor]
            A1_CONN[ConnectionManager]
            A1_PROC[ProcessManager]
        end

        subgraph "Agent 2"
            A2_APP[AgentApplication]
            A2_EXEC[TaskExecutor]
            A2_CONN[ConnectionManager]
            A2_PROC[ProcessManager]
        end

        subgraph "Agent N"
            AN_APP[AgentApplication]
            AN_EXEC[TaskExecutor]
            AN_CONN[ConnectionManager]
            AN_PROC[ProcessManager]
        end
    end

    subgraph "存储层"
        DB[(PostgreSQL)]
        CACHE[(Redis Cache)]
    end

    %% 交互关系
    USER --> WEB
    USER --> API
    WEB --> API_GATEWAY
    API --> API_GATEWAY

    API_GATEWAY --> AUTH
    API_GATEWAY --> SCHEDULER

    SRV_APP --> SCHEDULER
    SRV_APP --> BROKER
    SRV_APP --> API_GATEWAY

    SCHEDULER --> TASK_GEN
    SCHEDULER --> DISPATCHER
    TASK_GEN --> SCHED_BMC
    DISPATCHER --> BROKER

    BROKER --> LOAD_BAL
    BROKER --> WS_SERVER

    JOB_BMC --> DB
    TASK_BMC --> DB
    AGENT_BMC --> DB
    SCHED_BMC --> DB

    WS_SERVER --> A1_CONN
    WS_SERVER --> A2_CONN
    WS_SERVER --> AN_CONN

    A1_CONN --> A1_APP
    A1_APP --> A1_EXEC
    A1_EXEC --> A1_PROC

    A2_CONN --> A2_APP
    A2_APP --> A2_EXEC
    A2_EXEC --> A2_PROC

    AN_CONN --> AN_APP
    AN_APP --> AN_EXEC
    AN_EXEC --> AN_PROC

    %% 反向状态上报
    A1_CONN -->|状态上报| WS_SERVER
    A2_CONN -->|状态上报| WS_SERVER
    AN_CONN -->|状态上报| WS_SERVER

    style SRV_APP fill:#e3f2fd
    style A1_APP fill:#e8f5e8
    style A2_APP fill:#e8f5e8
    style AN_APP fill:#e8f5e8
    style DB fill:#ffcda8
    style CACHE fill:#fff3e0
```

## 核心组件交互时序

### 1. Agent 注册与连接建立时序

```mermaid
sequenceDiagram
    participant A as hetuflow-agent
    participant S as hetuflow-server
    participant DB as PostgreSQL
    participant L as LoadBalancer

    Note over A,S: Agent 启动与注册
    A->>L: WebSocket 连接请求
    L->>S: 路由连接
    S->>A: 连接建立成功

    A->>S: AgentRegisterRequest (capabilities, labels)
    S->>DB: 创建/更新 Agent 记录
    DB-->>S: Agent 信息保存成功
    S->>A: AgentRegisterResponse (success, config)

    Note over A,S: 心跳维持
    loop 定期心跳 (30s)
        A->>S: HeartbeatRequest (status, metrics)
        S->>DB: 更新 Agent 状态
        S->>A: HeartbeatResponse (commands, config)
    end

    Note over A,S: 任务拉取
    loop 任务拉取 (可配置间隔)
        A->>S: AcquireTaskRequest (capacity, labels)
        S->>DB: 查询匹配的任务
        DB-->>S: 返回待执行任务列表
        S->>A: AcquireTaskResponse (tasks, has_more)
    end
```

### 2. 任务调度与执行时序

```mermaid
sequenceDiagram
    participant User as 用户
    participant API as HTTP API
    participant SVC as SchedulerSvc
    participant DB as PostgreSQL
    participant BROKER as BrokerSvc
    participant AGENT as hetuflow-agent
    participant TASK as TaskExecutor

    Note over User,DB: 任务创建与调度
    User->>API: 创建 Job 和 Schedule
    API->>DB: 保存 Job 和 Schedule
    DB-->>API: 创建成功
    API-->>User: 返回 Job 信息

    Note over SVC,AGENT: 任务生成与分发
    SVC->>DB: TaskGenerationSvc 预生成任务
    DB-->>SVC: NOTIFY 'task_change'
    SVC->>SVC: SchedulerEngine 检查新任务
    SVC->>BROKER: 请求分发任务
    BROKER->>DB: 查询合适的 Agent
    DB-->>BROKER: 返回可用 Agent 列表
    BROKER->>AGENT: WebSocket 发送任务

    Note over AGENT,TASK: 任务执行
    AGENT->>AGENT: TaskScheduler 接收任务
    AGENT->>AGENT: ProcessManager 检查资源
    AGENT->>TASK: 创建 TaskExecutor
    TASK->>TASK: 执行任务命令
    TASK->>AGENT: 实时状态更新

    Note over AGENT,DB: 状态上报
    AGENT->>BROKER: TaskInstanceUpdated (status, metrics)
    BROKER->>DB: 更新 TaskInstance 状态
    DB-->>BROKER: 保存成功
    BROKER->>SVC: 任务状态变更通知
    SVC->>DB: 更新 Task 状态

    Note over TASK,AGENT: 任务完成
    TASK->>AGENT: TaskInstance 执行完成
    AGENT->>BROKER: 最终状态上报
    BROKER->>DB: 保存最终结果
    DB-->>API: 任务完成通知
    API-->>User: 任务执行结果
```

## 通信协议设计

### WebSocket 协议架构

hetuflow 使用 WebSocket 协议实现 Agent 与 Server 之间的全双工通信，具备以下优势：

- **全双工通信**：支持 Server 主动推送任务和 Agent 主动上报状态
- **网络穿透友好**：基于 HTTP/HTTPS，易于穿透防火墙和代理
- **连接复用**：长连接减少握手开销
- **实时性强**：低延迟的消息传递
- **自动重连**：Agent 支持断线重连和故障恢复

### 协议定义来源

所有通信协议定义统一在 [`hetuflow-core/src/protocol/`](../../../hetuflow-core/src/protocol/) 模块中管理：

- **消息格式**：[`event.rs`](../../../hetuflow-core/src/protocol/event.rs) 和 [`command.rs`](../../../hetuflow-core/src/protocol/command.rs)
- **数据模型**：[`models/`](../../../hetuflow-core/src/models/) 中的 Job/Task/TaskInstance 三层任务模型
- **类型定义**：[`types/mod.rs`](../../../hetuflow-core/src/types/mod.rs) 中的状态枚举和配置结构体

### 核心消息类型

基于最新代码实现，核心消息类型包括：

#### Agent -> Server 消息
- **AgentRegisterRequest**: Agent 注册请求 ([`protocol/agent.rs`](../../../hetuflow-core/src/protocol/agent.rs))
- **HeartbeatRequest**: 心跳请求 ([`protocol/heartbeat.rs`](../../../hetuflow-core/src/protocol/heartbeat.rs))
- **AcquireTaskRequest**: 任务拉取请求
- **TaskInstanceUpdated**: 任务实例状态更新

#### Server -> Agent 消息
- **AgentRegisterResponse**: Agent 注册响应
- **HeartbeatResponse**: 心跳响应
- **ScheduledTask**: 分发的任务
- **AgentCommand**: 服务器指令

### 核心消息类型

通过 `hetuflow-core` 定义的核心消息类型包括：

- **Agent 生命周期**：AgentRegisterRequest/Response、HeartbeatRequest
- **任务调度**：ScheduledTask、TaskInstanceUpdated、AcquireTaskRequest/Response
- **WebSocket 通信**：WebSocketEvent、WebSocketCommand
- **错误处理**：ErrorResponse、AckMessage 确认机制

### 使用方式

Server 和 Agent 都通过依赖 `hetuflow-core` 来获得一致的协议定义：

```toml
# Cargo.toml (Server 和 Agent 共同依赖)
[dependencies]
hetuflow-core = { workspace = true }
```

这确保了协议的版本一致性和类型安全。

## 数据模型设计

hetuflow 采用基于 **modelsql** ORM 的分层数据模型设计，确保数据库访问的类型安全、错误处理的一致性和代码的可维护性。

### 三层任务模型

```mermaid
graph TD
    subgraph "Job (作业定义)"
        J1[Job ID]
        J2[任务配置]
        J3[调度策略]
        J4[环境变量]
    end

    subgraph "Task (任务计划)"
        T1[Task ID]
        T2[Job ID 引用]
        T3[调度时间]
        T4[任务参数]
        T5[重试次数]
    end

    subgraph "TaskInstance (执行实例)"
        I1[Instance ID]
        I2[Task ID 引用]
        I3[Agent ID]
        I4[执行状态]
        I5[开始/结束时间]
        I6[执行结果]
    end

    J2 --> T2
    J3 --> T3
    T2 --> I2
    T4 --> I4
    T5 --> I6

    style J1 fill:#e3f2fd
    style T1 fill:#e8f5e8
    style I1 fill:#fff3e0
```

### 核心数据实体

基于最新代码实现 ([`models/mod.rs`](../../../hetuflow-core/src/models/mod.rs))：

- **[`SchedJob`](../../../hetuflow-core/src/models/job.rs)**: 作业静态定义（"做什么"），表 `sched_job`
- **[`SchedTask`](../../../hetuflow-core/src/models/task.rs)**: 待执行的计划（"何时做"），表 `sched_task`
- **[`SchedTaskInstance`](../../../hetuflow-core/src/models/task_instance.rs)**: 实际执行记录，表 `sched_task_instance`
- **[`SchedAgent`](../../../hetuflow-core/src/models/agent.rs)**: Agent 节点信息，表 `sched_agent`

### 关键数据流走向

#### 1. 任务创建与调度数据流

```mermaid
flowchart LR
    subgraph "输入层"
        USER[用户/API]
        EVENT[外部事件]
    end

    subgraph "服务层"
        API_GATEWAY[API Gateway]
        SVC[SchedulerSvc]
        TASK_GEN[TaskGenerationSvc]
    end

    subgraph "数据层"
        JOB_BMC[JobBmc]
        TASK_BMC[TaskBmc]
        SCHED_BMC[ScheduleBmc]
        DB[(PostgreSQL)]
    end

    subgraph "通知层"
        NOTIFY[PG NOTIFY]
        BROKER[BrokerSvc]
    end

    subgraph "执行层"
        WS[WebSocket]
        AGENTS[Agent集群]
    end

    USER --> API_GATEWAY
    EVENT --> API_GATEWAY
    API_GATEWAY --> SVC
    SVC --> JOB_BMC
    SVC --> SCHED_BMC

    TASK_GEN --> SCHED_BMC
    TASK_GEN --> TASK_BMC

    JOB_BMC --> DB
    TASK_BMC --> DB
    SCHED_BMC --> DB

    DB -.->|NOTIFY 'task_change'| NOTIFY
    NOTIFY --> BROKER
    BROKER --> WS
    WS --> AGENTS

    style USER fill:#e3f2fd
    style DB fill:#ffcda8
    style AGENTS fill:#e8f5e8
```

#### 2. 任务执行与状态更新数据流

```mermaid
flowchart TB
    subgraph "Agent 执行层"
        AGENT[hetuflow-agent]
        EXECUTOR[TaskExecutor]
        PROCESS[ProcessManager]
    end

    subgraph "状态收集层"
        COLLECTOR[状态收集器]
        METRICS[指标收集]
    end

    subgraph "通信层"
        WS_CONN[WebSocket连接]
        BROKER[BrokerSvc]
    end

    subgraph "服务层"
        SVC[SchedulerSvc]
        DB_SVC[TaskInstanceSvc]
    end

    subgraph "数据层"
        TASK_BMC[TaskInstanceBmc]
        TASK_INST_BMC[TaskInstanceBmc]
        DB[(PostgreSQL)]
    end

    subgraph "监控层"
        MONITOR[监控系统]
        ALERT[告警系统]
    end

    EXECUTOR --> PROCESS
    PROCESS --> COLLECTOR
    COLLECTOR --> METRICS

    METRICS --> WS_CONN
    WS_CONN --> BROKER
    BROKER --> SVC
    BROKER --> DB_SVC

    SVC --> TASK_BMC
    DB_SVC --> TASK_INST_BMC

    TASK_BMC --> DB
    TASK_INST_BMC --> DB

    DB -.->|状态变更| MONITOR
    MONITOR --> ALERT

    style AGENT fill:#e8f5e8
    style DB fill:#ffcda8
    style MONITOR fill:#fff3e0
```

## 任务状态流转

### 任务状态机

基于最新代码实现 ([`types/mod.rs`](../../../hetuflow-core/src/types/mod.rs#78-93))：

```mermaid
stateDiagram-v2
    [*] --> Pending: 任务创建
    Pending --> Locked: Server获取任务
    Locked --> Dispatched: 分发给Agent
    Dispatched --> Running: Agent开始执行
    Running --> Failed: 执行失败
    Failed --> WaitingRetry: 未达重试限制
    WaitingRetry --> Running: 重试执行
    Running --> Succeeded: 执行成功
    Running --> Cancelled: 任务取消
    Failed --> [*]: 达到重试限制
    Succeeded --> [*]: 任务完成
    Cancelled --> [*]: 任务结束
```

### Agent 能力与标签匹配

基于 [`types/mod.rs`](../../../hetuflow-core/src/types/mod.rs#14-41) 的实现，支持多种任务类型：

```mermaid
graph TD
    subgraph "调度类型 (ScheduleKind)"
        CRON[Cron定时作业]
        INTERVAL[间隔定时作业]
        DAEMON[守护进程作业]
        EVENT[事件驱动作业]
        FLOW[流程任务]
    end

    subgraph "执行命令 (ExecuteCommand)"
        BASH[Bash]
        UV[Uv]
        PYTHON[Python]
        NODE[Node.js]
        NPX[Npx]
        CARGO[Cargo]
        JAVA[Java]
    end

    subgraph "Agent 标签匹配"
        LABELS[任务标签]
        CAPABILITIES[Agent能力]
        RESOURCES[资源限制]
    end

    CRON --> LABELS
    INTERVAL --> LABELS
    EVENT --> LABELS
    DAEMON --> LABELS
    FLOW --> LABELS

    LABELS --> CAPABILITIES
    CAPABILITIES --> RESOURCES

    style CRON fill:#e3f2fd
    style BASH fill:#e8f5e8
    style LABELS fill:#fff3e0
```

## 现代化技术特性

### 1. 架构设计优势

基于最新的代码实现，hetuflow 具备以下现代化架构特性：

- **Application 容器模式**: 使用 [`fusion-core::Application`](../../../crates/libs/fusion-core/src/) 统一管理服务依赖和生命周期
- **类型安全 ORM**: 基于 [`modelsql`](../../../crates/libs/modelsql/) 的全程类型安全数据库操作
- **分层错误处理**: `modelsql::SqlError → fusion_core::DataError` 的分层错误转换机制
- **WebSocket 全双工通信**: 支持服务器推送和 Agent 上报的双向实时通信
- **强一致性存储**: 基于 PostgreSQL 事务保证的 ACID 特性

### 2. 性能优化特性

- **连接复用**: Agent 与 Server 保持长连接，减少握手开销
- **数据库通知**: 基于 PostgreSQL LISTEN/NOTIFY 的实时任务通知机制
- **批量处理**: 支持任务的批量分发和状态更新
- **资源限制**: 支持内存、CPU、执行时间等资源限制 ([`types/mod.rs#ResourceLimits`](../../../hetuflow-core/src/types/mod.rs#218-240))
- **智能调度**: 基于标签匹配和负载均衡的任务分发算法

### 3. 可靠性保障

- **自动重连**: Agent 支持断线重连和故障恢复
- **任务重试**: 支持可配置的重试策略和退避算法
- **状态持久化**: 所有状态信息持久化到 PostgreSQL，避免数据丢失
- **事务保证**: 利用数据库事务确保操作的原子性和一致性
- **监控告警**: 完整的任务执行监控和错误告警机制

### 4. 开发体验优化

- **代码生成**: 使用 [`modelsql::Fields`](../../../crates/libs/modelsql/) 宏自动生成 CRUD 操作
- **字段级更新**: 支持字段掩码的部分更新操作，减少数据传输
- **过滤器 DSL**: 提供类型安全的查询过滤器系统
- **编译时检查**: 全程类型安全，编译时发现错误
- **文档完整**: 完整的 API 文档和代码示例

## 系统总结

hetuflow 是一个基于 Rust 2024 Edition 构建的现代化分布式任务调度系统，通过 WebSocket 全双工通信、PostgreSQL 强一致性存储、modelsql 类型安全 ORM 等现代技术栈，实现了高性能、高可靠性的任务调度能力。

### 核心价值主张

1. **类型安全保障**: 从数据库操作到通信协议的全程类型安全
2. **现代化架构**: Application 容器 + BMC 分层 + WebSocket 通信的统一架构
3. **高性能通信**: WebSocket 全双工通信 + PostgreSQL 实时通知
4. **强一致性存储**: 基于 PostgreSQL ACID 事务的数据一致性保证
5. **网络友好**: 基于 HTTP/HTTPS 的 WebSocket，易于穿透防火墙
6. **开发体验**: 代码生成、编译时检查、完整文档的现代化开发体验

### 技术架构亮点

- **统一的协议定义**: 通过 `hetuflow-core` 确保通信协议的一致性和类型安全
- **三层任务模型**: Job → Task → TaskInstance 的清晰业务模型
- **智能任务分发**: 基于标签匹配和负载均衡的智能调度算法
- **完整的监控体系**: 从任务创建到执行完成的全链路监控
- **灵活的扩展能力**: 支持多种调度类型和执行命令的插件化设计

hetuflow 特别适合需要 **高可靠性**、**强一致性**、**类型安全** 的分布式任务调度场景，是企业级任务调度系统的理想选择。
