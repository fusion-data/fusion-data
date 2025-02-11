# 调度系统

## 目标

设计一款可靠、易用、可扩展、轻量的通用分布式任务调度系统。暂且将此系统称为fusion-scheduler，融合各类业务的调度系统之意：

- 可靠：采用分布式设计，避免任务丢失，支持对失败任务的自动或手动重试
- 易用：支持固定间隔、固定延迟、CRON 表达式、API 触发、工作流、MapReduce 等调度方式，满足各类业务需求
- 可扩展：支持动态添加、删除任务、动态修改任务配置，支持动态添加、删除任务执行器、动态修改任务执行器配置
- 轻量：占用资源少

## 概念

### 架构设计

fusion-scheduler 作为一款分布工调度系统，架构上采用 Multi-Scheduler + Multi-Worker 的架构设计。多个 Scheduler 节点用于任务调度控制，比如：根据分组获取任务列表、分发任务到 Worker 节点执行、……；而多个 Worker 节点是用于执行任务，Worker 节点可以部署在多台机器上以提供更高的并发任务执行能力。

![Scheduler 和 Worker 分离](imgs/scheduler-Scheduler和Worker分离.drawio.svg)

对于 Scheduler 和 Worker，设计为一种节点角色，也既一个节点可以同时作为 Scheduler 和 Worker 节点运行，比如在本机开发时只需要启动一个服务即可拥有所有功能。而在线上环境，可以根据负载和作业执行时需要的特殊性来部署合适数量的 Scheduler 和 Worker 节点。

![Scheduler 和 Worker 集成](imgs/scheduler-Scheduler和Worker集成.drawio.svg)

fusion-scheduler 采用最小化设计原则，不依赖第三方中间件，采用 Rust 语言和 PostgreSQL（以后简称：PG）数据库开发。PG 将作为集群选主、任务定义管理、任务执行状态等集群状态数据存储库。

### 集群管理

集群节点有两种角色：Scheduler 和 JobWorker。

#### Scheduler

Scheduler 角色的节点负责调度作业任务管理，将任务分发到 JobWorker 节点执行。Scheduler 角色节点的主要简化执行流程可如下描述：

1. `Scheduler` 根据 `namespace` 获取所有在指定时间前需要触发的 `TriggerDefinition`（触发器定义）
2. 通过 `TriggerDefinition` 及关联的 `ProcessDefinition`（流程定义）生成 `ProcessInstance`（流程实例），并由一个 **时间轮** 来定时触发流程实例执行
3. 当 `ProcessInstance` 被触发时根据流程中定义的 `Task` 生成一个或多个 `TaskInstance`，并由 `Scheduler` 将其分发到某一个 `JobWorker` 节点执行

#### JobWorker

JobWorker 负责接收作业任务并执行（由 `JobWorker` 实现）。一个 `JobWorker` 必须与 Scheduler 通信，除此外并无其它限制。可以选择将 JobWorker 作为微服务来实现，也可以作为经典3层架构应用程序的一部分、或作为 Lamdba JobWorker 角色节点的主要简化执行流程可如下描述：

1. `JobWorker` 根据 `namespace` 注册自身到相应的 Scheduler 角色节点（`Scheduler`）
2. Worker 注册成功后，通过一个 gRPC 服务端流 API 等待接收 Job 任务
3. 收到一个 TaskJob 后执行，并将执行状态、执行结果及时送回 Scheduler 角色节点（`Scheduler`）

### 流程（任务）

```plantuml
@startuml
class ProcessDefinition {
  id： Uuid
}
class TriggerDefinition {
  id： Uuid
}
class ProcessInstance {
  id： Uuid
  process_definition_id： Uuid
  trigger_definition_id： Option<Uuid>
}
class TaskInstance {
  id： Uuid
  process_instance_id： Uuid
}
class TaskJob {
  id： Uuid
  task_id： Uuid
}

TriggerDefinition -- ProcessDefinition ： 触发 >
ProcessDefinition -- ProcessInstance ： 生成 >
ProcessInstance -- TaskInstance ： 包含 >
TaskInstance -- TaskJob ： 执行 >
@enduml
```

#### 流程概念

##### 流程定义： `ProcessDefinition`

`fusion-scheduler` 管理的单位是流程（`ProcessDefinition`），描述一个流程的执行规则，包括流程名称、执行参数、执行数据、标签等。每个流程可以由多个任务（`Task`）组成，任务依赖由 DAG 进行管理。

##### 触发器定义： `TriggerDefinition`

触发器定义。描述一个任务触发规则，包括触发方式、触发参数、……

触发器定义通常关联到一个 `ProcessDefinition`，当触发器定义被触发时，会生成一个 `ProcessInstance`。

##### 流程实例： `ProcessInstance`

流程实例。描述一个流程的运行状态，包括流程开始时间、结束时间、执行状态、执行参数、执行数据、……一个流程实例可以由关联的 `TriggerDefinition` 触发执行，也可以通过 API 编程触发执行

##### 任务实例： `TaskInstance`

流程中实际执行的任务，它可能是一段 Rust/Java/Python/Shell/SQL 代码，也可能是一个 HTTP API 调用等，由 `TaskType` 来决定。

##### 任务作业： `TaskJob`

任务作业。TaskInstance 的一次实际执行（记录），当任务因失败或其它原因被重试时也会生成一个 TaskJob。`TaskJob` 将由（被调度到）Job Worker 节点执行。

### 流程与任务

可以看到，`TaskInstance` 更像传统的作业调度系统的概念，而 `ProcessDefinition` 更像 BPMN 中的一个业务流程（*当一个流程只有一个任务时，就类似于传统的作业调度系统，比如：Quartz 的 Job*）。这是 `fusion-scheduler` 分布式通用任务调度系统架构设计的核心。它既可用于定义任务作业，也可以用于编排业务流程微服务，还可以用于工作流和审批流调度。

## 技术架构

技术上，fusion-scheduler 采用了 Rust 语言和 PostgreSQL 作为主要开发语言和数据库。计划用到的主要库和框架有：

- [tokio](https://crates.io/crates/tokio)： 异步执行库
- [axum](https://crates.io/crates/axum)/[tower](https://crates.io/crates/tower)/[hyper](https://crates.io/crates/hyper)： HTTP 服务框架
- [tonic](https://crates.io/crates/tonic)/[prost](https://crates.io/crates/prost)/[nice-grpc-web](https://github.com/deeplay-io/nice-grpc/tree/master/packages/nice-grpc-web)（gRPC-Web 客户端库）： gRPC 通信
- [sqlx](https://crates.io/crates/sqlx)/[sea-query](https://crates.io/crates/sea-query)： 数据库访问
- [react](https://react.dev/)/[ant.desigin](https://ant-design.antgroup.com/)： 前端框架和 UI 库
- [PostgreSQL](https://www.postgresql.org/)： 数据存储、集群状态
- [sqlite](https://sqlite.org/)： 计划用于 Worker 节点本地缓存和持久化数据（日志）存储

调度系统集群将主要有 3 个（服务）角色：

- `Master`： 集群主节点，负责平衡 `Scheduler` 可调度的 `Namespace` 列表，监听并/检查名节点状态，包括：Scheduler 和 Worker 节点。
- `Scheduler`： 负责任务调度，生成任务作业，管理任务作业状态，管理任务作业执行结果。
- `Worker`： 接受任务并在工作节点上执行。

### Master

同一时间段内，只有一个 `Master` 节点活跃，其它节点的 Master 处于 Standby 状态。Master 的工作职责有：

- 检查 Scheduler 活跃状态，更新 Scheduler 关联的 Namespace 节点列表
- 检查未分配 Namespace 节点（比如新增）并分配给 Scheduler

### Scheduler

#### Scheduler 主要职责

1. 从数据库获取待执行触发器，并生成流程实例
2. 从数据库获取待执行流程实例，并生成任务作业，将作业分发到 Worker 节点执行
3. 接收 API 任务执行命令，以可编程形式手动触发作业
4. 接收 Worker 节点的作业执行结果，更新作业状态和结果
5. 处理失败任务
6. 接收任务执行日志等，……

#### Scheduler 调度任务流程

1. Scheduler init
2. register to pg sched_scheduler
3. 尝试向 sched_namespace 表感兴趣的 namespace 写入自己的 scheduler_id
   - 成功则继续
   - 失败则说明相关 Scheduler 节点已经存在，则将当前 Scheduler 转为容错备用模式，等待尝试重新关联
4. 遍历有效的 trigger_definition，计算并生成 process_instance
5. 遍历有效的 process_instance，生成 process_task
6. 遍历待执行的 process_task，生成 task_job
7. 分发 task_job 到相关的 job_worker

#### 高可用

对于每一个（流程）任务，都将先在数据库中插入一条记录（`ProcessTask`），再由 `Scheduler` 分发到对应的 `Worker` 节点执行。这样，就算 `Worker` 节点挂掉或因定时问题错过任务执行，后续 `Scheduler` 也可以对 `ProcessTask` 重新分发或继续执行。

#### 按 namespace 隔离任务

namespace 可作为任务的分组，以方便管理类似业务的所有流程。

## 小结

市面上已有很多优秀的分布式调度库/框架/系统，如：[Quartz](https://www.quartz-scheduler.org/)、[DolphinScheduler](https://dolphinscheduler.apache.org/)、[Airflow](https://airflow.apache.org/)、[PowerJob](http://www.powerjob.tech/)、[Xxl-Job](https://www.xuxueli.com/xxl-job/) 等。这些系统都给了我设计上的思路和帮助，在此表达感谢。
