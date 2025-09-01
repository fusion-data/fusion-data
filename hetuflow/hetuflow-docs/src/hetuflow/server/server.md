# hetuflow-server

hetuflow-server 是 hetuflow 系统的核心协调节点，负责任务编排、分发、状态管理、权限控制、Web 管理界面和 API 服务。

## 主要功能

- 提供 Web 管理界面和 HTTP API 服务
- 管理与所有 Agent 的 WebSocket 通信
- 负责任务的编排、分发和状态管理
- 实现用户权限管理和系统监控

## 内部模块架构

`hetuflow-server` 项目目录参考如下：

```
src/
├── application.rs   # Application 容器，管理服务依赖
├── lib.rs          # 库入口文件
├── endpoint/       # API 端点
│   ├── api/
│   │   └── v1/     # API v1 版本
│   └── mod.rs      # 模块定义
├── service/         # 业务服务模块
│   ├── gateway_svc.rs    # 网关服务
│   ├── agent_svc.rs      # Agent 管理服务
    ├── scheduler_svc.rs  # 调度服务
    ├── agent_manager.rs  # Agent 管理器
    ├── task_generation_service.rs   # 任务生成服务（预生成/事件驱动）
    ├── task_poller.rs    # 任务轮询器
    ├── load_balancer.rs  # 负载均衡器
    └── mod.rs            # 模块定义
```

## 核心架构组件

- **Application**: 应用程序容器，统一管理所有服务和依赖关系，提供依赖注入功能
- **ModelManager**: 数据库连接和操作管理器，统一管理数据库访问
- **DbBmc (Database Basic Model Controller)**: 数据库操作抽象层，提供类型安全的 CRUD 操作
- **Service 层**: 业务逻辑层，使用 ModelManager 和 DbBmc 进行数据库操作
- **ultimate_core::DataError**: 统一的错误处理机制，将 modelsql::SqlError 转换为应用层错误

## 核心特性

- 基于 Axum 框架的 HTTP API 服务
- 使用现代化数据库技术栈：
  - **ultimate-core::Application**: 依赖注入容器和应用生命周期管理
  - **modelsql::ModelManager**: 数据库连接池和操作管理
  - **modelsql::base::DbBmc**: 统一的数据库操作抽象层
  - **modelsql::SqlError →fusion_core::DataError**: 分层错误处理机制
  - **sea-query**: 类型安全的 SQL 查询构建器
- 基于 tokio-tungstenite 的高性能 WebSocket 服务
- 支持任务的动态配置和热更新
- 实现任务执行的容错和重试机制

## 配置参数

基于 `ultimate-core` crate 的 `UltimateConfigRegistry` 配置注册器加载配置文件，配置文件路径默认是项目根目录 `resources/app.toml`，可以通过 `ULTIMATE_CONFIG_PATH` 环境变量指定配置文件路径。实现配置文件见: [app.toml](../../../fusion/hetuflow-server/resources/app.toml)

## Application 与服务层实现

### Application 容器架构

调度器采用现代化的 `Application` 容器模式，统一管理所有服务和组件依赖，确保资源管理和生命周期控制的一致性。

一些组件依赖

- [ConnectionManager](./server-gateway.md#connectionmanager): 连接管理器，负责管理所有 Agent 与网关的 WebSocket 连接
- [MessageHandler](./server-gateway.md#messagerouter): 消息路由，负责将网关事件和调度器事件路由到相应的处理程序
- [SchedulerSvc](./server-scheduler.md#schedulersvc): 调度服务，负责任务编排、分发和状态管理
- [GatewaySvc](./server-gateway.md#gatewaysvc): 网关服务，负责处理 Agent 与网关的 WebSocket 通信

```rust
use fusion_core::{DataError, application::Application};
use fusion_db::DbPlugin;
use modelsql::ModelManager;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::{mpsc, RwLock, Mutex};
use std::collections::HashMap;

// Hetuflow 应用容器
#[derive(Clone)]
pub struct ServerApplication {
  mm: ModelManager,
  scheduler_svc: Arc<SchedulerSvc>,
  gateway_svc: Arc<GatewaySvc>,
  task_generation_svc: Arc<TaskGenerationSvc>,
  agent_manager: Arc<AgentManager>,
  is_leader: Arc<AtomicBool>,
}
```

`ServerApplication` 完整的代码实现见： [application.rs](../../../fusion/hetuflow-server/src/application.rs)

### 核心组件架构

见 `architecture.md` 的 [组件关系图](../architecture.md#组件关系图) 部分
