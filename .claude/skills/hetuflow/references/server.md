# hetuflow-server

调度服务器：Broker、Scheduler、Service、Endpoint。

## Imports

```rust
// Service
use hetuflow_server::service::{TaskSvc, JobSvc, ScheduleSvc, AgentSvc};

// BMC
use hetuflow_server::infra::bmc::{TaskBmc, JobBmc, ScheduleBmc, AgentBmc};

// Broker
use hetuflow_server::broker::{MessageBroker, BrokerConfig};

// Scheduler
use hetuflow_server::scheduler::{TaskScheduler, SchedulerConfig};

// Endpoint
use hetuflow_server::endpoint::api::v1::*;
```

## Service Layer

### TaskSvc

```rust
use hetusql::ModelManager;
use hetus::core::DataError;

pub struct TaskSvc {
    mm: ModelManager,
}

impl TaskSvc {
    pub fn new(mm: ModelManager) -> Self {
        Self { mm }
    }

    pub async fn create(&self, mut task_data: TaskForCreate) -> Result<Uuid, DataError> {
        let task_id = task_data.id.unwrap_or_else(Uuid::now_v7);

        self.mm.transaction(|mm| async move {
            // 获取 Job 配置
            let job = JobBmc::find_by_id(&mm, task_data.job_id).await?
                .ok_or_else(|| DataError::not_found("Job not found"))?;

            // 继承 Job 配置
            task_data.config = Some(job.config.clone());

            // 创建任务
            TaskBmc::insert(&mm, task_data).await?;
            Ok(task_id)
        }).await
    }

    pub async fn update_status(&self, task_id: Uuid, status: TaskStatus) -> Result<(), DataError> {
        let update = TaskForUpdate {
            status: Some(status),
            update_mask: Some(FieldMask::new(vec!["status".to_string(), "updated_at".to_string()])),
            ..Default::default()
        };
        TaskBmc::update_by_id(&self.mm, task_id, update).await
    }

    pub async fn list(&self, filter: Option<TaskFilter>, page: Option<Page>) -> Result<PageResult<SchedTask>, DataError> {
        TaskBmc::list(&self.mm, filter, page).await
    }
}
```

### JobSvc

```rust
pub struct JobSvc {
    mm: ModelManager,
}

impl JobSvc {
    pub async fn create(&self, job: JobForCreate) -> Result<Uuid, DataError> {
        let job_id = job.id.unwrap_or_else(Uuid::now_v7);
        JobBmc::insert(&self.mm, job).await?;
        Ok(job_id)
    }

    pub async fn enable(&self, job_id: Uuid) -> Result<(), DataError> {
        let update = JobForUpdate {
            status: Some(JobStatus::Enabled),
            ..Default::default()
        };
        JobBmc::update_by_id(&self.mm, job_id, update).await
    }

    pub async fn disable(&self, job_id: Uuid) -> Result<(), DataError> {
        let update = JobForUpdate {
            status: Some(JobStatus::Disabled),
            ..Default::default()
        };
        JobBmc::update_by_id(&self.mm, job_id, update).await
    }
}
```

## Broker

### MessageBroker

```rust
use hetuflow_server::broker::{MessageBroker, BrokerConfig};

pub struct MessageBroker {
    connections: DashMap<Uuid, WebSocketConnection>,
    config: BrokerConfig,
}

impl MessageBroker {
    pub async fn register_agent(&self, agent_id: Uuid, conn: WebSocketConnection) {
        self.connections.insert(agent_id, conn);
    }

    pub async fn send_command(&self, agent_id: &Uuid, command: Command) -> Result<()> {
        let conn = self.connections.get(agent_id)
            .ok_or_else(|| Error::AgentNotFound)?;
        conn.send(command).await
    }

    pub async fn broadcast(&self, command: Command) {
        for conn in self.connections.iter() {
            let _ = conn.send(command.clone()).await;
        }
    }
}
```

## Scheduler

### TaskScheduler

```rust
use hetuflow_server::scheduler::{TaskScheduler, SchedulerConfig};
use croner::Cron;

pub struct TaskScheduler {
    mm: ModelManager,
    broker: Arc<MessageBroker>,
    timer: HierarchicalHashWheelTimer,
}

impl TaskScheduler {
    pub async fn start(&self) -> Result<()> {
        // 加载所有启用的 Schedule
        let schedules = ScheduleBmc::list_enabled(&self.mm).await?;

        for schedule in schedules {
            self.schedule_task(schedule).await?;
        }

        Ok(())
    }

    pub async fn schedule_task(&self, schedule: SchedSchedule) -> Result<()> {
        match schedule.schedule_kind {
            ScheduleKind::Cron => {
                let cron = Cron::new(&schedule.cron_expression)?;
                let next_run = cron.next_after(now_utc())?;

                self.timer.schedule_at(next_run, move || {
                    // 创建任务
                    let task = TaskForCreate::from_schedule(&schedule);
                    TaskBmc::insert(&mm, task).await?;
                });
            }
            ScheduleKind::Interval => {
                let interval = Duration::from_secs(schedule.interval_seconds);
                self.timer.schedule_fixed_rate(interval, move || {
                    let task = TaskForCreate::from_schedule(&schedule);
                    TaskBmc::insert(&mm, task).await?;
                });
            }
            _ => {}
        }
        Ok(())
    }
}
```

## API Endpoints

### Jobs API

```rust
use utoipa_axum::router::OpenApiRouter;

pub fn routes() -> OpenApiRouter<ServerApplication> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(query_jobs))
        .routes(utoipa_axum::routes!(create_job))
        .routes(utoipa_axum::routes!(get_job))
        .routes(utoipa_axum::routes!(update_job))
        .routes(utoipa_axum::routes!(enable_job))
        .routes(utoipa_axum::routes!(disable_job))
        .routes(utoipa_axum::routes!(delete_job))
}

#[utoipa::path(get, path = "/page", tag = "Jobs")]
async fn query_jobs(
    job_svc: JobSvc,
    Query(filter): Query<JobFilter>,
    Query(page): Query<Page>,
) -> WebResult<PageResult<SchedJob>> {
    let result = job_svc.list(Some(filter), Some(page)).await?;
    ok_json!(result)
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

### Tasks API

```rust
pub fn routes() -> OpenApiRouter<ServerApplication> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(query_tasks))
        .routes(utoipa_axum::routes!(get_task))
        .routes(utoipa_axum::routes!(cancel_task))
        .routes(utoipa_axum::routes!(retry_task))
}

#[utoipa::path(post, path = "/item/{id}/cancel", tag = "Tasks")]
async fn cancel_task(
    Path(id): Path<Uuid>,
    task_svc: TaskSvc,
) -> WebResult<()> {
    task_svc.update_status(id, TaskStatus::Cancelled).await?;
    ok_json!({})
}
```

## Best Practices

1. **事务**: 涉及多表操作时使用 `mm.transaction()`
2. **状态管理**: Job 状态变更通过 enable/disable 方法
3. **调度器**: 使用 hierarchical_hash_wheel_timer 高效调度
4. **WebSocket**: Broker 管理所有 Agent 连接

## Examples from Codebase

- `hetuflow/hetuflow-server/src/service/task_svc.rs` - Task Service
- `hetuflow/hetuflow-server/src/scheduler/mod.rs` - Scheduler
- `hetuflow/hetuflow-server/src/endpoint/api/v1/jobs.rs` - Jobs API
