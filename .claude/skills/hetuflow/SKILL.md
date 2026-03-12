---
name: hetuflow
description: Hetuflow distributed task scheduler, broker, scheduler, agent, task queue, cron job, workflow scheduling, 分布式任务调度
globs:
  - "hetuflow/**/*.rs"
---

# Hetuflow - 分布式任务调度系统

## Quick Reference

| Module | Import | Key Types |
|--------|--------|-----------|
| Core | `use hetuflow_core::*;` | `SchedTask`, `TaskConfig`, `TaskStatus`, `ScheduleKind` |
| Protocol | `use hetuflow_core::protocol::*;` | `CommandKind`, `EventKind` |
| Server | `use hetuflow_server::*;` | `TaskSvc`, `JobSvc`, `SchedulerSvc` |
| Agent | `use hetuflow_agent::*;` | `Agent`, `TaskExecutor` |

## Core Types

### Task Status
```rust
pub enum TaskStatus {
    Pending = 1,     // 等待分发
    Doing = 10,      // 执行中
    Failed = 90,     // 失败
    Cancelled = 99,  // 已取消
    Succeeded = 100, // 成功
}
```

### Schedule Kind
```rust
pub enum ScheduleKind {
    Cron = 1,      // Cron 定时任务
    Interval = 2,  // 间隔任务
    Event = 3,     // 事件驱动
    Flow = 4,      // 流式任务
    Daemon = 5,    // 守护进程
}
```

### Task Config
```rust
pub struct TaskConfig {
    pub timeout: u32,
    pub max_retries: u32,
    pub retry_interval: u32,
    pub cmd: ExecuteCommand,  // Bash, Uv, Python, Node, Npx, Cargo, Java
    pub args: Vec<String>,
    pub capture_output: bool,
    pub labels: Option<Labels>,
    pub resource_limits: Option<ResourceLimits>,
}
```

## Core Patterns

### Service Layer
```rust
use hetuflow_server::service::TaskSvc;
use hetusql::ModelManager;

pub struct TaskSvc {
    mm: ModelManager,
}

impl TaskSvc {
    pub fn new(mm: ModelManager) -> Self {
        Self { mm }
    }

    pub async fn create_task(&self, task_data: TaskForCreate) -> Result<Uuid, DataError> {
        let task_id = task_data.id.unwrap_or_else(Uuid::now_v7);

        self.mm.transaction(|mm| async move {
            let job = JobBmc::find_by_id(&mm, task_data.job_id).await?;
            TaskBmc::insert(&mm, task_data).await?;
            Ok(task_id)
        }).await
    }
}
```

### API Endpoint
```rust
use utoipa_axum::router::OpenApiRouter;

pub fn routes() -> OpenApiRouter<ServerApplication> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(query_jobs))
        .routes(utoipa_axum::routes!(create_job))
        .routes(utoipa_axum::routes!(get_job))
        .routes(utoipa_axum::routes!(enable_job))
        .routes(utoipa_axum::routes!(disable_job))
}

#[utoipa::path(post, path = "/item", tag = "Jobs")]
async fn create_job(
    job_svc: JobSvc,
    Json(input): Json<JobForCreate>,
) -> WebResult<IdUuidResult> {
    let id = job_svc.create(input).await?;
    ok_json!(IdUuidResult::from(id))
}
```

### Protocol Messages
```rust
use hetuflow_core::protocol::{CommandKind, EventKind};

// Commands (Server -> Agent)
pub enum CommandKind {
    Shutdown = 1,
    UpdateConfig = 2,
    ClearCache = 3,
    FetchMetrics = 4,
    AgentRegistered = 5,
    TaskAcquired = 6,
    CancelTask = 7,
}

// Events (Agent -> Server)
pub enum EventKind {
    Ack = 1,
    Nack = 2,
    RegisterAgent = 3,
    Heartbeat = 4,
    AcquireTask = 5,
    TaskInstanceChanged = 6,
    LogMessage = 7,
}
```

## References (按需加载)

- [core](references/core.md) - models, protocol, types
- [server](references/server.md) - broker, scheduler, service, endpoint
- [agent](references/agent.md) - 任务执行代理

## Related Skills
- `hetu`: 核心库模式
- `sql-database`: BMC/Service 层
- `tokio-axum-patterns`: 异步和 Web 模式
