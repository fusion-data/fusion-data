# hetuflow-core

核心共享库：数据模型、通信协议、类型定义。

## Imports

```rust
use hetuflow_core::{
    models::{
        SchedTask, SchedTaskInstance, TaskFilter, TaskForCreate,
        TaskForQuery, TaskForUpdate, TaskInstanceFilter, TaskInstanceForCreate,
        SchedJob, JobFilter, JobForCreate, JobForUpdate,
        SchedSchedule, ScheduleFilter, ScheduleForCreate,
        SchedAgent, AgentFilter,
    },
    types::{ScheduleKind, TaskStatus, TaskInstanceStatus, JobStatus, AgentStatus},
    protocol::{CommandKind, EventKind, Command, Event},
};
```

## Data Models

### SchedTask

```rust
pub struct SchedTask {
    pub id: Uuid,
    pub job_id: Uuid,
    pub namespace_id: String,
    pub priority: i32,
    pub status: TaskStatus,
    pub schedule_id: Option<Uuid>,
    pub scheduled_at: DateTime<FixedOffset>,
    pub schedule_kind: ScheduleKind,
    pub completed_at: Option<DateTime<FixedOffset>>,
    pub environment: Option<serde_json::Value>,
    pub parameters: serde_json::Value,
    pub config: TaskConfig,
    pub retry_count: i32,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: Option<DateTime<FixedOffset>>,
}
```

### SchedJob

```rust
pub struct SchedJob {
    pub id: Uuid,
    pub namespace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: JobStatus,
    pub schedule_kind: ScheduleKind,
    pub config: TaskConfig,
    pub labels: Option<Labels>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: Option<DateTime<FixedOffset>>,
}
```

### TaskConfig

```rust
pub struct TaskConfig {
    pub timeout: u32,
    pub max_retries: u32,
    pub retry_interval: u32,
    pub cmd: ExecuteCommand,
    pub args: Vec<String>,
    pub capture_output: bool,
    pub labels: Option<Labels>,
    pub resource_limits: Option<ResourceLimits>,
}

pub enum ExecuteCommand {
    Bash,
    Uv,
    Python,
    Node,
    Npx,
    Cargo,
    Java,
}
```

### SchedSchedule

```rust
pub struct SchedSchedule {
    pub id: Uuid,
    pub job_id: Uuid,
    pub namespace_id: String,
    pub name: String,
    pub cron_expression: Option<String>,
    pub interval_seconds: Option<u32>,
    pub timezone: String,
    pub enabled: bool,
    pub start_at: Option<DateTime<FixedOffset>>,
    pub end_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
}
```

## Status Enums

### TaskStatus
```rust
pub enum TaskStatus {
    Pending = 1,     // 等待分发
    Doing = 10,      // 执行中
    Failed = 90,     // 失败
    Cancelled = 99,  // 已取消
    Succeeded = 100, // 成功
}
```

### JobStatus
```rust
pub enum JobStatus {
    Created = 1,
    Disabled = 99,
    Enabled = 100,
}
```

### AgentStatus
```rust
pub enum AgentStatus {
    Idle = 10,
    Busy = 20,
    Connecting = 30,
    Disconnecting = 31,
    Offline = 90,
    Error = 99,
    Online = 100,
}
```

### ScheduleKind
```rust
pub enum ScheduleKind {
    Cron = 1,      // Cron 定时任务
    Interval = 2,  // 间隔任务
    Event = 3,     // 事件驱动
    Flow = 4,      // 流式任务
    Daemon = 5,    // 守护进程
}
```

## Protocol

### Command (Server -> Agent)
```rust
pub enum CommandKind {
    Shutdown = 1,
    UpdateConfig = 2,
    ClearCache = 3,
    FetchMetrics = 4,
    AgentRegistered = 5,
    TaskAcquired = 6,
    CancelTask = 7,
}

pub struct Command {
    pub kind: CommandKind,
    pub payload: Option<serde_json::Value>,
}
```

### Event (Agent -> Server)
```rust
pub enum EventKind {
    Ack = 1,
    Nack = 2,
    RegisterAgent = 3,
    Heartbeat = 4,
    AcquireTask = 5,
    TaskInstanceChanged = 6,
    LogMessage = 7,
}

pub struct Event {
    pub kind: EventKind,
    pub agent_id: Uuid,
    pub payload: Option<serde_json::Value>,
}
```

## Filters

```rust
use hetusql_macros::FilterNodes;

#[derive(Debug, Default, Deserialize, FilterNodes)]
pub struct TaskFilter {
    pub id: Option<OpValUuid>,
    pub job_id: Option<OpValUuid>,
    pub namespace_id: Option<OpValString>,
    pub status: Option<OpValInt32>,
    pub schedule_kind: Option<OpValInt32>,
}

#[derive(Debug, Default, Deserialize, FilterNodes)]
pub struct JobFilter {
    pub id: Option<OpValUuid>,
    pub namespace_id: Option<OpValString>,
    pub name: Option<OpValString>,
    pub status: Option<OpValInt32>,
}
```

## Best Practices

1. **状态机**: TaskStatus 是状态机，只能按特定路径转换
2. **命名空间**: 使用 namespace_id 隔离不同租户的任务
3. **重试策略**: 通过 TaskConfig.max_retries 和 retry_interval 控制
4. **超时**: 设置合理的 timeout 防止任务无限运行

## Examples from Codebase

- `hetuflow/hetuflow-core/src/models/task.rs` - Task 模型
- `hetuflow/hetuflow-core/src/models/job.rs` - Job 模型
- `hetuflow/hetuflow-core/src/protocol/mod.rs` - 协议定义
