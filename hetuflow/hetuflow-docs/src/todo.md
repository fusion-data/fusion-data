# Hetuflow 后端实现待办任务

## 一、核心共享库 (hetuflow-core)

### 1.1 通信协议实现

- [*] **WebSocketMessage 定义** - 实现统一消息包装器，包含 message_id、timestamp、message_kind、payload、metadata
- [*] **MessageKind 枚举** - 实现完整的消息类型枚举，涵盖错误处理、Agent 生命周期、任务调度、文件传输
- [*] **Agent 注册协议** - 实现 AgentRegisterRequest/Response、AgentCapabilities、AgentConfig 数据结构
- [*] **心跳协议** - 实现 HeartbeatRequest/Response、AgentMetrics、AgentCommand 数据结构
- [*] **任务调度协议** - 实现 DispatchTaskRequest、TaskInstanceUpdate、TaskConfig、TaskMetrics 数据结构

### 1.2 数据模型定义

- [*] **作业模型** - 实现 Job 数据结构定义，包含作业静态配置
- [*] **任务模型** - 实现 Task 数据结构定义，表示具体执行计划
- [*] **任务实例模型** - 实现 TaskInstance 数据结构定义，记录实际执行
- [*] **调度类型枚举** - 实现 ScheduleKind 枚举：Cron、Time、Daemon、Event、Flow
- [*] **状态枚举** - 实现 TaskInstanceStatus、AgentStatus 等状态定义

### 1.3 序列化与验证

- [*] **JSON 序列化支持** - 为所有协议消息实现 serde Serialize/Deserialize
- [*] **数据库支持** - 为必要的枚举实现 sqlx::Type (feature = "with-db")
- [*] **消息验证** - 实现消息格式验证和错误处理机制

### 1.4 数据实体和访问模型设计

- [*] **JobEntity 定义** - 实现作业实体数据结构，对应 sched_job 表
- [*] **TaskEntity 定义** - 实现任务实体数据结构，对应 sched_task 表
- [*] **TaskInstanceEntity 定义** - 实现任务实例实体数据结构，对应 sched_task_instance 表
- [*] **AgentEntity 定义** - 实现 Agent 实体数据结构，对应 sched_agent 表
- [*] **ServerEntity 定义** - 实现服务器实体数据结构，对应 sched_server 表
- [*] **ScheduleEntity 定义** - 实现调度实体数据结构，对应 sched_schedule 表
- [*] **ForCreate/ForUpdate/Filter 结构** - 为每个实体实现创建、更新、过滤数据结构

## 二、服务端 (hetuflow-server)

### 2.1 应用容器架构

- [*] **ServerApplication 实现** - 基于 ultimate-core::Application 的应用容器，管理所有服务依赖
- [*] **服务注册与启动** - 实现服务的依赖注入、生命周期管理和优雅关闭
- [*] **配置管理** - 实现基于 app.toml 的配置加载和验证

### 2.2 BMC 层

- [*] **JobBmc 实现** - 实现 Job 的数据库基础操作控制器
- [*] **TaskBmc 实现** - 实现 Task 的数据库基础操作控制器
- [*] **TaskInstanceBmc 实现** - 实现 TaskInstance 的数据库基础操作控制器
- [*] **AgentBmc 实现** - 实现 Agent 的数据库基础操作控制器
- [*] **ServerBmc 实现** - 实现 Server 的数据库基础操作控制器
- [*] **ScheduleBmc 实现** - 实现 Schedule 的数据库基础操作控制器

### 2.3 网关服务 (Gateway)

- [*] **GatewaySvc 实现** - 核心网关服务，管理 WebSocket 连接和消息路由
- [*] **ConnectionManager 实现** - WebSocket 连接管理器，维护 Agent 连接状态
- [*] **MessageHandler 实现** - 消息处理器，处理上下行消息分发
- [*] **WebSocket 服务器** - 基于 Axum 的 WebSocket 服务器实现
- [*] **连接健康检查** - 实现连接状态监控和异常处理

### 2.4 API 端点层

- [*] **API 错误处理** - 实现统一的 API 错误响应格式
- [*] **Agent 管理 API** - 实现 /api/v1/agents 相关端点
- [*] **Job 管理 API** - 实现 /api/v1/jobs 相关端点
- [*] **Task 管理 API** - 实现 /api/v1/tasks 相关端点
- [*] **TaskInstance 管理 API** - 实现 /api/v1/task-instances 相关端点
- [*] **系统监控 API** - 实现 /api/v1/system 相关端点
- [*] **Gateway API** - 实现 /api/v1/gateway 相关端点
<!-- - [ ] **API 文档** - 生成 OpenAPI/Swagger 文档 -->

### 2.5 调度服务 (Scheduler)

- [*] **SchedulerSvc 实现** - 核心调度服务，处理任务编排和分发逻辑
- [*] **TaskPoller 实现** - 轮询调度器，定时查询待处理任务
- [*] **LoadBalancer 实现** - Namespace 负载均衡器。负载均衡机制基于 **namespace_id** 进行 namespace_id 分发，通过 **leader server** 自动管理各个 server 实例与 namespace_id 的绑定关系，实现 namespace_id 的智能分发和负载均衡
- [*] **TaskGenerationSvc 实现** - 任务生成服务，根据 Job 配置生成 Task 实例
- [*] **领导者选举** - 实现分布式领导者选举机制: `start_leader_and_follower_loop`，基于 PG 数据表的分布式锁。相关算法说明： [基于 PostgreSQL 的分布式锁设计](server/distributed_lock.md)
- ~~[*] **任务分发策略** - 实现基于 Agent 能力和负载的任务分发算法~~: **采用 poll 模式，由 Agent 主动拉取任务，Server 不向 Agent 分派任务**
- [*] **任务状态管理** - 实现任务状态流转和生命周期管理
- [ ] **重试机制** - 实现任务失败重试和超时处理

### 2.6 辅助服务

- [ ] **AgentManager 实现** - Agent 管理器，维护 Agent 状态和能力信息
- [ ] **负载均衡器** - 实现基于 namespace_id 的负载均衡算法
- [ ] **健康检查服务** - 实现系统健康状态监控

### 2.7 安全与认证

- [ ] **API 认证机制** - 实现 JWT 或 API Key 认证
- [ ] **权限控制** - 实现基于角色的访问控制 (RBAC)
- [ ] **Agent Token 验证** - 实现 Agent 连接的 Token 验证

### 2.8 监控与日志

- [ ] **结构化日志** - 使用 tracing 实现结构化日志记录
- [ ] **指标收集** - 实现系统性能指标收集
- [ ] **链路追踪** - 实现分布式链路追踪

## 三、客户端 (hetuflow-agent)

### 3.1 应用容器架构

- [ ] **AgentApplication 实现** - 基于 ultimate-core::Application 的 Agent 应用容器
- [ ] **配置管理** - 实现 Agent 配置加载和验证
- [ ] **服务启动** - 实现 Agent 各组件的启动和生命周期管理

### 3.2 连接管理

- [ ] **ConnectionManager 实现** - WebSocket 连接管理器，处理与 Server 的通信
- [ ] **自动重连机制** - 实现连接断开后的自动重连
- [ ] **心跳维持** - 实现定期心跳保持连接活跃
- [ ] **消息队列** - 实现本地消息队列缓存待发送消息

### 3.3 任务调度

- [ ] **TaskScheduler 实现** - 精调度器，基于 hierarchical_hash_wheel_timer 实现高精度定时
- [ ] **Cron 解析** - 使用 croner 解析 Cron 表达式
- [ ] **本地定时器** - 实现基于时间轮的本地定时任务触发
- [ ] **任务同步** - 实现与 Server 的任务状态同步

### 3.4 任务执行

- [ ] **TaskExecutor 实现** - 任务执行器，管理本地任务进程
- [ ] **进程管理** - 实现子进程创建、监控和控制
- [ ] **并发控制** - 实现最大并发任务数限制
- [ ] **资源监控** - 实现任务执行过程中的资源使用监控
- [ ] **输出捕获** - 实现标准输出/错误的实时捕获和上报
- [ ] **超时处理** - 实现任务执行超时的检测和处理

### 3.5 状态管理

- [ ] **任务状态上报** - 实现任务状态变更的实时上报
- [ ] **系统指标上报** - 实现 Agent 系统指标的定期上报
- [ ] **能力声明** - 实现 Agent 能力的动态声明和更新

### 3.6 日志管理

- [ ] **LogManager 实现** - 任务执行日志的收集和管理
- [ ] **日志转发** - 实现任务日志向 Server 的实时转发
- [ ] **本地日志** - 实现本地日志文件的管理和轮转

## 四、数据库与存储

### 4.1 数据库结构

- [*] **DDL 脚本完善** - 完善 hetuflow-ddl.sql 中的表结构定义
- [*] **索引优化** - 为查询性能添加必要的数据库索引
- [*] **约束定义** - 实现数据完整性约束和外键关系

## 五、配置与部署

### 5.1 配置文件

- [ ] **Server 配置模板** - 完善 hetuflow-server 的 app.toml 配置模板
- [ ] **Agent 配置模板** - 完善 hetuflow-agent 的 app.toml 配置模板
- [ ] **环境变量支持** - 实现敏感配置通过环境变量覆盖

### 5.2 部署配置

- [ ] **Docker 镜像** - 创建 Server 和 Agent 的 Docker 镜像构建配置
- [ ] **Kubernetes 配置** - 创建 K8s 部署配置文件
- [ ] **系统服务配置** - 创建 systemd 服务配置文件

### 5.3 数据迁移

- [ ] **迁移脚本** - 实现数据库版本迁移脚本
- [ ] **数据初始化** - 实现系统基础数据的初始化脚本

## 六、测试与质量保证

### 6.1 单元测试

- [ ] **核心组件测试** - 为所有核心组件编写单元测试
- [ ] **BMC 层测试** - 为数据库操作层编写测试
- [ ] **协议测试** - 为通信协议实现测试

### 6.2 集成测试

- [ ] **端到端测试** - 实现 Server-Agent 完整交互的集成测试
- [ ] **负载测试** - 实现系统负载能力测试
- [ ] **故障测试** - 实现故障恢复和容错测试

### 6.3 文档

- [ ] **API 文档** - 生成完整的 API 接口文档
- [ ] **部署文档** - 编写系统部署和运维文档
- [ ] **开发文档** - 编写开发者指南和代码结构说明

## 七、性能优化

### 7.1 数据库优化

- [ ] **查询优化** - 优化高频查询的 SQL 性能
- [ ] **连接池配置** - 优化数据库连接池配置
- [ ] **批量操作** - 实现批量插入和更新操作

### 7.2 内存优化

- [ ] **内存使用分析** - 分析和优化内存使用情况
- [ ] **缓存策略** - 实现合理的内存缓存策略

### 7.3 网络优化

- [ ] **消息确认机制** - 实现消息的 ACK 确认和重试逻辑
- [ ] **消息压缩** - 实现 WebSocket 消息压缩
- [ ] **连接复用** - 优化网络连接的复用策略

## 八、运维监控

### 8.1 监控指标

- [ ] **业务指标** - 定义和实现关键业务指标监控
- [ ] **性能指标** - 实现系统性能指标监控
- [ ] **错误监控** - 实现错误率和异常监控

### 8.2 告警机制

- [ ] **告警规则** - 定义系统告警规则和阈值
- [ ] **通知渠道** - 实现多种告警通知渠道

### 8.3 运维工具

- [ ] **状态检查工具** - 实现系统状态检查命令行工具
- [ ] **数据修复工具** - 实现数据一致性检查和修复工具

---

## 实施优先级建议

## 详细里程碑与 Issue 清单

### 阶段一：核心基础（高优先级）

#### 里程碑 1.1：Core 通信协议落地（hetuflow-core）

- [*] Issue 1.1.1 定义 WebSocketMessage 与 MessageKind
  - 目录：fusion/hetuflow-core/src/protocol/
  - 验收标准：
    - 序列化/反序列化通过单元测试（含 snake_case JSON 示例）
    - message_id 使用 UUID v7，timestamp 使用 Epoch Millis
  - 依赖：serde、serde_json、uuid、chrono
  - 预估：M
- [*] Issue 1.1.2 Agent 注册协议与心跳协议数据结构
  - 目录：fusion/hetuflow-core/src/protocol/agent/
  - 验收标准：
    - AgentRegisterRequest/Response、HeartbeatRequest/Response 可 JSON 往返
    - AgentCapabilities、AgentMetrics 具备基础字段（CPU、内存、并发度）
  - 依赖：Issue 1.1.1
  - 预估：M
- [*] Issue 1.1.3 任务调度协议结构体
  - 目录：fusion/hetuflow-core/src/protocol/task/
  - 验收标准：
    - DispatchTaskRequest、TaskInstanceUpdate、TaskConfig、TaskMetrics 可 JSON 往返
    - 提供最小端到端示例（doc tests）
  - 依赖：Issue 1.1.1
  - 预估：M
- [*] Issue 1.1.4 with-db 特性与枚举 sqlx::Type 实现
  - 目录：fusion/hetuflow-core/src/types/
  - 验收标准：
    - 打开 feature = "with-db" 后，编译通过且基本 CRUD 测试可映射
    - TaskInstanceStatus、AgentStatus、ScheduleKind 实现 sqlx::Type(Postgres)
  - 依赖：sqlx、postgres-types
  - 预估：M

#### 里程碑 1.2：数据模型与 BMC 框架（hetuflow-server）

- [*] Issue 1.2.1 建表与迁移脚本初始化
  - 目录：scripts/migrations/ 与 fusion/hetuflow-server/resources/
  - 验收标准：
    - 执行迁移后生成 sched_job、sched_task、sched_task_instance、sched_agent、sched_server、sched_schedule 表
    - 必要索引创建完成（任务状态、更新时间、namespace 等）
  - 依赖：PostgreSQL 环境
  - 预估：M
- [*] Issue 1.2.2 实体定义与 ForCreate/ForUpdate/Filter
  - 目录：fusion/hetuflow-core/src/models/
  - 验收标准：
    - 各实体及三类 DTO 编译通过，包含基本校验
    - 与 modelsql 框架适配基本 CRUD
  - 依赖：Issue 1.2.1
  - 预估：M
- [*] Issue 1.2.3 BMC 层 CRUD 骨架
  - 目录：fusion/hetuflow-server/src/infra/bmc/
  - 验收标准：
    - JobBmc、TaskBmc、TaskInstanceBmc、AgentBmc、ServerBmc、ScheduleBmc 提供 create/get/query/update/delete 基本方法
    - 单元测试覆盖常见路径（成功/不存在/约束冲突）
  - 依赖：Issue 1.2.2
  - 预估：L

#### 里程碑 1.3：ServerApplication 容器与配置

- [*] Issue 1.3.1 应用容器与依赖注入
  - 目录：fusion/hetuflow-server/src/app/
  - 验收标准：
    - Application 启动/停止生命周期完整，服务可注册与获取
    - 优雅关闭（SIGINT/SIGTERM）
  - 依赖：tokio、ultimate-core
  - 预估：M
- [*] Issue 1.3.2 配置加载与校验
  - 目录：fusion/hetuflow-server/resources/app.toml、src/config/
  - 验收标准：
    - 从 app.toml 加载，支持环境变量覆盖
    - 错误配置给出清晰错误
  - 依赖：config、serde
  - 预估：S

#### 里程碑 1.4：基础 WebSocket 连接管理（Gateway MVP）

- [ ] Issue 1.4.1 WebSocket 端点与连接接入
  - 目录：fusion/hetuflow-server/src/gateway/
  - 验收标准：
    - 暴露 /ws 端点，可接受连接，完成 ping/pong
    - 记录简单连接日志与连接计数
  - 依赖：axum、tokio-tungstenite
  - 预估：M
- [ ] Issue 1.4.2 初步认证占位
  - 验收标准：
    - 通过 Header/Query 传递 token 的占位校验
    - 可切换到严格校验的扩展点
  - 依赖：Issue 1.4.1
  - 预估：S

---

### 阶段二：核心功能（高优先级）

#### 里程碑 2.1：Gateway 完整化

- [ ] Issue 2.1.1 ConnectionManager
  - 验收标准：
    - 连接注册/注销，按 agent_id 查询与广播
    - 并发安全（RwLock/ShardMap），压测 1k 连接下稳定
  - 依赖：Issue 1.4.1
  - 预估：M
- [ ] Issue 2.1.2 MessageHandler 与 ACK/重试
  - 验收标准：
    - 上下行消息分类路由；支持消息 ID、ACK、重试（指数退避）
    - 重试上限与死信日志
  - 依赖：Issue 2.1.1、Issue 1.1.x 协议
  - 预估：L
- [ ] Issue 2.1.3 认证与会话
  - 验收标准：
    - Token 校验可插拔（本地/远程 IAM）
    - 会话过期与踢下线
  - 依赖：Issue 2.1.1
  - 预估：M

#### 里程碑 2.2：Scheduler 基础能力

- [ ] Issue 2.2.1 TaskPoller 周期轮询
  - 验收标准：
    - 可配置周期；仅拉取待处理任务
    - 指标：拉取数量、延迟
  - 依赖：BMC、配置
  - 预估：M
- [ ] Issue 2.2.2 TaskGenerationSvc 任务生成服务
  - 验收标准：
    - 使用 SELECT FOR UPDATE SKIP LOCKED 实现并发安全领取
    - 在多实例下无重复领取（集成测试）
  - 依赖：Postgres、sqlx
  - 预估：M
- [ ] Issue 2.2.3 Leader 选举（简版）
  - 验收标准：
    - 基于 DB 锁/记录的 leader 抢占与续约
    - leader 变更触发安全切换
  - 依赖：Issue 1.3.1、数据库
  - 预估：M
- [ ] Issue 2.2.4 任务分发策略（基础）
  - 验收标准：
    - 按 Agent 在线与能力简单匹配
    - 失败回退与重试
  - 依赖：Gateway、BMC
  - 预估：M

#### 里程碑 2.3：Agent 基础能力

- [ ] Issue 2.3.1 AgentApplication 与配置
  - 目录：fusion/hetuflow-agent/src/app/
  - 验收标准：
    - 启停生命周期完整，配置加载
  - 依赖：ultimate-core、config
  - 预估：S
- [ ] Issue 2.3.2 连接管理与心跳
  - 验收标准：
    - 启动后自动连接 server，保持心跳
    - 断线自动重连
  - 依赖：Issue 2.3.1、协议
  - 预估：M
- [ ] Issue 2.3.3 接收任务与回报状态（最小执行器）
  - 验收标准：
    - 能接收 DispatchTaskRequest 并回报开始/成功/失败
    - 先使用 mock 执行器（sleep/echo）
  - 依赖：Gateway、协议
  - 预估：M

#### 里程碑 2.4：基础 REST API

- [ ] Issue 2.4.1 Axum 路由与中间件
  - 验收标准：
    - CORS、日志、中间件链路基本完善
  - 依赖：Issue 1.3.x
  - 预估：S
- [ ] Issue 2.4.2 Job/Agent 基础端点
  - 验收标准：
    - Job: create/query/get，Agent: query
    - 单元测试 + 简要文档
  - 依赖：BMC
  - 预估：M
- [ ] Issue 2.4.3 OpenAPI/Swagger 基础
  - 验收标准：
    - 通过 utoipa 或类似方案生成基本文档
  - 依赖：Issue 2.4.1
  - 预估：S

---

### 阶段三：高级功能（中优先级）

#### 里程碑 3.1：基于 namespace_id 的负载均衡

- [ ] Issue 3.1.1 Namespace 管理与绑定
  - 验收标准：
    - 支持 namespace_id 创建、权重配置、手动绑定
  - 依赖：BMC、Gateway
  - 预估：M
- [ ] Issue 3.1.2 加权轮询与健康度
  - 验收标准：
    - 加权轮询路由；结合心跳健康度调整权重
    - 基准测试显示分布合理
  - 依赖：Issue 3.1.1
  - 预估：M

#### 里程碑 3.2：高级调度策略

- [ ] Issue 3.2.1 任务生成策略与 Cron 解析
  - 验收标准：
    - 能基于 Cron 生成 Task，误差 < 1s
  - 依赖：croner 或等价库
  - 预估：M
- [ ] Issue 3.2.2 事件/流程型调度占位
  - 验收标准：
    - 提供事件触发与流程编排的接口占位与最小实现
  - 依赖：SchedulerSvc
  - 预估：M

#### 里程碑 3.3：API 覆盖完善

- [ ] Issue 3.3.1 Task/TaskInstance 端点集
  - 验收标准：
    - Task: CRUD、启停；TaskInstance: 查询、重试/终止
  - 依赖：BMC
  - 预估：L
- [ ] Issue 3.3.2 连接/系统监控端点
  - 验收标准：
    - /connections、/system 指标输出 JSON
  - 依赖：Gateway、metrics
  - 预估：M

#### 里程碑 3.4：监控与日志

- [ ] Issue 3.4.1 tracing 结构化日志
  - 验收标准：
    - 关键路径日志 + trace_id 贯通 Server/Agent
  - 依赖：tracing、tracing-subscriber
  - 预估：S
- [ ] Issue 3.4.2 指标收集与导出
  - 验收标准：
    - 暴露 Prometheus 指标；核心指标齐全（任务吞吐、延迟、失败率、连接数）
  - 依赖：prometheus/metrics 库
  - 预估：M

---

### 阶段四：完善优化（中低优先级）

#### 里程碑 4.1：性能优化

- [ ] Issue 4.1.1 数据库性能优化
  - 验收标准：
    - 高频查询 explain 优化；关键索引到位
  - 依赖：运行基线数据
  - 预估：M
- [ ] Issue 4.1.2 并发与内存优化
  - 验收标准：
    - 压测下无明显内存泄露与锁竞争热点
  - 依赖：bench/pprof
  - 预估：M

#### 里程碑 4.2：测试与文档覆盖

- [ ] Issue 4.2.1 端到端集成测试
  - 验收标准：
    - Server 与 Agent 端到端用例（创建 Job/Task、分发、执行、回报）
  - 依赖：docker-compose 测试环境
  - 预估：L
- [ ] Issue 4.2.2 文档完善
  - 验收标准：
    - API、部署、开发者指南完整且与实现一致
  - 依赖：前述模块完成度
  - 预估：M

#### 里程碑 4.3：部署与运维

- [ ] Issue 4.3.1 Docker 镜像与 Compose
  - 验收标准：
    - 一键启动最小可运行集群（server+agent+postgres）
  - 依赖：前述模块基本可用
  - 预估：M
- [ ] Issue 4.3.2 Kubernetes 与 systemd
  - 验收标准：
    - 提供 K8s 部署示例与 systemd 服务文件
  - 依赖：Issue 4.3.1
  - 预估：M

---

### 全局要求

- 每个 Issue 均需：
  - 完成 Definition of Done：代码+测试+文档（如适用）
  - 代码注释到函数级别；遵循 2 空格缩进
  - 安全与错误处理遵循统一规范（thiserror/anyhow、tracing）
- 里程碑验收：
  - 演示用例或脚本，展示端到端路径通畅
  - 指标/日志具备可观测性
