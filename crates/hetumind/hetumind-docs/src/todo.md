# Hetumind 开发计划与待办任务 (TODO)

本文档根据项目设计文档，制定详细的开发计划和待办任务列表。

## 开发阶段总览

1.  **Phase 0: 项目设置与基础 (Project Setup & Foundation)** - 搭建项目骨架，配置开发环境和 CI/CD。
2.  **Phase 1: MVP - 核心引擎与本地运行 (Core Engine & Local Runner)** - 实现核心的工作流执行能力和本地开发调试工具。
3.  **Phase 2: 功能增强 - API、前端与 AI 集成 (API, Frontend & AI Integration)** - 构建服务端 API、可视化前端以及核心的 AI 编排能力。
4.  **Phase 3: 生产就绪 - 云原生与可扩展性 (Cloud-Native & Scalability)** - 实现 Lambda 部署、高级监控、插件系统和整体系统加固。
5.  **Phase 4: 未来展望与社区生态 (Future & Community)** - 规划高级功能和社区建设。

---

## Todo

- [ ] 重构: [`hetumind`](crates/hetumind/hetumind) 为 [`hetumind-studio`](crates/hetumind/hetumind-studio)
- [ ] 重构: 将 user/role/permission 等相关的代码抽离出来，独立一个 `hetumind-iam` 服务
- [ ] 重构: 将作业任务（Job Task）提取出来，独立一个 `hetumind-task` 服务
- [ ] 考虑使用模板引擎作为 expression 的增强，比如：`minijinja`

---

## Phase 0: 项目设置与基础 (Project Setup & Foundation)

- [x] **项目结构**
  - [x] 初始化 Rust Workspace `hetumid`
  - [x] 创建核心 crates: `hetumind`, `hetumind-nodes`, `hetumind-core`
  - [x] 创建前端项目目录 `hetumind-web`
  - [x] 创建文档目录 `docs` 和设计文档目录 `datum`
- [x] **依赖管理**
  - [x] 配置根 `Cargo.toml` 和各个 crate 的依赖（如 `tokio`, `serde`, `uuid`, `chrono` 等）
  - [x] 在 `hetumind-web` 中初始化 `package.json` 并添加 Vue, Vue Flow, OpenTiny, Pinia, Vue Router 等依赖
- [x] **配置管理**
  - [x] 设计并实现统一的配置加载机制（支持 `app.toml` 文件和环境变量），由 [UltimateConfig](../../crates/ultimates/ultimate-core/src/configuration/model/ultimate_config.rs) 实现
- [x] **构建**
  - [x] 设置 `rust-toolchain.toml`，统一 Rust 版本
  - [x] 配置 Dockerfile 用于后端服务的容器化

---

## Phase 1: MVP - 核心引擎与本地运行 (Core Engine & Local Runner)

### 核心 (`hetumind-core`, `hetumind::runtime`)

- [ ] **核心类型系统 (`hetumind-core`)**
  - [x] 定义所有 ID 类型 (`WorkflowId`, `NodeId`, etc.) 使用 `uuid::Uuid`
  - [x] 定义 `ExecutionStatus`, `NodeExecutionStatus`, `NodeKind` 等核心枚举
  - [x] 实现核心数据结构 `Workflow`, `WorkflowNode`, `Connection`，并添加 `serde` 支持
  - [x] 实现执行相关结构 `Execution`, `NodeExecution`, `ExecutionContext`, `ExecutionData`
  - [x] 定义核心 Trait: `NodeExecutor`, `TriggerExecutor`, `WorkflowEngine`
  - [x] 使用 `thiserror` 设计并实现完整的错误处理类型 `GuixuError`, `WorkflowExecutionError`, etc.
- [x] **基础执行引擎 (`hetumind::runtime`)**
  - [x] 创建 `WorkflowEngineImpl` 实现 `WorkflowEngine` Trait
  - [x] 实现一个基于拓扑排序的图执行逻辑
    - [x] _子任务:_ 从 `Workflow` 结构构建依赖图 (e.g., `HashMap<NodeId, Vec<NodeId>>`)
    - [x] _子任务:_ 实现拓扑排序算法以确定节点执行顺序
  - [x] 实现节点间数据传递和上下文管理
    - [x] _子任务:_ 实现逻辑以汇集父节点的输出，作为当前节点的输入
  - [x] 实现基本的错误处理（任何节点失败则停止整个工作流）

### 节点系统 (`hetumind-nodes`)

- [x] **节点注册表**
  - [x] 在 `hetumind-runtime` 中实现 `NodeRegistry` 用于注册和查找节点执行器
- [ ] **实现标准节点**
  - [x] _子任务:_ 实现 `HttpRequestNode`，发送基本的 HTTP 请求
  - [x] _子任务:_ 实现 `IfNode`，进行简单的条件判断
  - [x] _子任务:_ 实现 `MergeNode`，合并多个分支的数据流
  - [x] _子任务:_ 实现 `SetNode`，用于设置或修改数据
  - [x] _子任务:_ 实现 `LoopOverItems`，循环控制节点，用于将大量数据分割成较小的批次进行逐批处理。
- [x] 基于已实现的 5 个节点实现应用于 `NodeRegistry` 的完整工作流定义和使用的 example

### 数据库 (`hetumind::db`)

- [x] **数据库 Schema 设计**
  - [x] 使用 `sqlx-cli` 初始化迁移目录: `sqlx migrate add initial_schema`
  - [x] 编写初始 Schema 迁移文件 (`001_initial_schema.sql`)
  - [x] 创建 `users`, `workflows`, `nodes`, `connections` 表，字段遵循 `05-database-design.md`
- [x] **数据访问层 (Repository)**
  - [x] `UserEntity` 及 CRUD
  - [x] `WorkflowEntity` 及 CRUD
  - [x] `ExecutionEntity` 及 CRUD

### 本地运行器 (`hetumind-cli`)

- [x] **CLI 框架**
  - [x] 使用 `clap` 搭建命令结构: `workflow new|list|validate|run|export`
- [x] **本地执行功能**
  - [x] 实现 `workflow run <file>` 命令
  - [x] _子任务:_ 实现从 JSON/YAML 文件加载工作流定义
  - [x] _子任务:_ 调用 `LocalExecutionEngine` 执行工作流
  - [x] _子任务:_ 使用 `tracing` 和 `tracing-subscriber` 在控制台输出执行日志
- [x] **工作流验证**
  - [x] 实现 `workflow validate <file>` 命令，检查节点类型是否存在、连接是否有效等

---

## Phase 2: 功能增强 - API、前端与 AI 集成 (API, Frontend & AI Integration)

### 后端 API (`hetumind::api`)

- [x] **API 框架**
  - [x] 使用 `Axum` 搭建 Web 服务，设置 `main.rs`
  - [x] 设计并实现 `Hetumind`，用于共享数据库连接池、引擎等
  - [x] 配置 `tower-http` 的 `CorsLayer`, `TraceLayer` 等中间件
- [x] **认证与授权**
  - [x] 实现 `SignSvc` (JWT, `argon2` 密码哈希)
  - [x] 实现用户注册和登录的 API Endpoints
  - [x] 实现 `WebAuth`，从 `Authorization` 头解析 `Ctx` 并注入
- [ ] **工作流 API**
  - [x] 实现工作流的 `CRUD` endpoints (`/api/v1/workflows`)
  - [ ] 实现 `/execute`, `/activate`, `/deactivate` 等操作接口
- [ ] **执行管理 API**
  - [ ] 实现 `executions` 相关的 `GET` 接口，支持分页和过滤
  - [ ] 实现 `POST /executions/{id}/cancel` 和 `.../retry`
- [ ] **凭证管理**
  - [ ] 在数据库中添加 `credentials` 表
  - [ ] 实现凭证的加密存储（例如使用 `magic-crypt`）和管理的 API

### 前端 (`hetumind-web`)

- [ ] **项目基础**
  - [ ] 使用 `Vue Router` 设置路由 (`/signin`, `/dashboard`, `/workflows/:id`)
  - [ ] 使用 `Pinia` 管理全局状态 (用户信息, Auth Token)
  - [ ] 封装一个 `axios` 实例作为 API 客户端，自动附加 Auth Token
- [ ] **用户认证页面**
  - [ ] 实现登录、注册表单页面
- [ ] **工作流设计器**
  - [ ] 集成 `Vue Flow`作为画布
  - [ ] _子任务:_ 实现一个可拖拽的节点面板，从 API 获取可用节点列表
  - [ ] _子任务:_ 实现节点参数配置面板，根据节点定义动态渲染表单
  - [ ] _子任务:_ 实现工作流的保存（创建/更新）和从 API 加载功能
- [ ] **执行历史视图**
  - [ ] 实现执行历史列表页面，调用执行查询 API
  - [ ] 实现一个模态框或侧边栏，用于显示单次执行的详细日志和每个节点的输入/输出

### 触发器与高级节点

- [ ] **触发器节点**
  - [ ] `WebhookTriggerNode`: 实现一个统一的 Webhook 入口 `/webhooks/{webhook_id}`，根据 ID 查找并触发工作流
  - [ ] `ScheduleTriggerNode`: 在服务启动时加载并启动所有激活的定时任务
- [ ] **AI 编排节点 (`hetumind-nodes`)**
  - [ ] `LLMCallNode`: 实现对主流 LLM API (Deepseek, Tongyi) 的调用
  - [ ] `PromptTemplateNode`: 使用类似 `Handlebars` 或 `liquid` 的模板引擎渲染 Prompt
- [ ] **数据库节点**
  - [ ] 实现 `DatabaseQueryNode` (PostgreSQL)，能使用已存储的凭证连接数据库

### 运行时增强 (`hetumind::runtime`)

- [ ] **任务调度器**
  - [ ] 实现包含任务队列、等待/运行任务管理的 `TaskScheduler`，以支持更复杂的执行逻辑（如循环、等待）
- [ ] **并发控制**
  - [ ] 实现基于 `tokio::sync::Semaphore` 的 `ConcurrencyController`，限制并发执行数量

---

## Phase 3: 生产就绪 - 云原生与可扩展性 (Cloud-Native & Scalability)

### Lambda 执行 (`hetumind-lambda`)

- [ ] **Lambda 运行时**
  - [ ] 创建 `hetumind-lambda` crate，集成 `lambda_runtime`
  - [ ] 实现 `function_handler`，路由到不同的执行逻辑
  - [ ] 实现冷启动优化：延迟初始化、复用全局状态
- [ ] **无状态执行引擎**
  - [ ] 实现 `LambdaExecutionEngine`，支持在超时前分批执行节点
  - [ ] 设计 `LambdaRequest` 和 `LambdaResponse` 结构，用于函数间调用
- [ ] **状态管理**
  - [ ] 实现 `StateManager`，使用 `aws-sdk-s3` 将 `ExecutionState` 序列化后存入 S3
  - [ ] `ExecutionState` 中需包含完整的执行状态，如各节点状态、中间数据等
- [ ] **部署**
  - [ ] 编写 `template.yaml` (AWS SAM) 或 `main.tf` (Terraform) 脚本用于一键部署

### 系统加固

- [ ] **监控与可观测性**
  - [ ] 集成 `tracing-opentelemetry` 和 `opentelemetry-otlp` 导出链路追踪数据
  - [ ] 集成 `metrics-exporter-prometheus` 并暴露 `/metrics` endpoint
- [ ] **错误处理与容错**
  - [ ] 在工作流设置中添重试策略（次数、间隔）
  - [ ] 实现 `Error Trigger` 节点，用于捕获上游节点的错误
- [ ] **数据库优化**
  - [ ] 编写迁移文件，为 `executions` 和 `node_executions` 表按月或按周设置分区
  - [ ] 分析慢查询并添加或优化查询索引

### CI/CD

- [ ] 创建 Github Actions CI 流程 (`ci.yml`)
  - [ ] `cargo fmt --check` (代码格式化检查)
  - [ ] `cargo clippy -- -D warnings` (代码质量检查)
  - [ ] `cargo test` (单元/集成测试)
  - [ ] `cargo build --release` (编译检查)

### 插件化与扩展性

- [ ] **WASM 节点支持**
  - [ ] 实现 `WasmNode`，集成 `extism` 作为 WASM 运行时
  - [ ] 设计安全的 WASM 代码执行沙箱，限制文件、网络等访问
- [ ] **节点开发 SDK**
  - [ ] 创建 `hetumind-node-sdk` crate，导出核心 Trait 和数据结构
  - [ ] 编写详细的节点开发文档和示例项目

---

## Phase 4: 未来展望与社区生态 (Future & Community)

- [ ] **高级 AI 功能**
  - [ ] `VectorStoreNode`: 集成 `pgvector`，实现文档的 Upsert 和 Similarity Search
  - [ ] `RAGNode`: 创建一个封装了"检索-增强-生成"流程的复合节点
  - [ ] `AIAgentNode`: 基于 `rig` 或类似框架实现具备工具调用能力的 Agent 逻辑
- [ ] **高级工作流功能**
  - [ ] _子任务:_ 添加 `workflow_versions` 表
  - [ ] _子任务:_ API 支持版本列表查看和回滚
  - [ ] _子任务:_ 实现工作流的导入/导出 (JSON/YAML) 功能
- [ ] **多租户/团队协作**
  - [ ] 设计团队和基于角色的访问控制 (RBAC) 数据模型
  - [ ] 在所有 API 查询中强制执行租户和权限隔离
- [ ] **社区建设**
  - [ ] 完善官方文档和各模块的 `README.md`
  - [ ] 创建一个包含常用工作流示例的模板市场
- [ ] **实时协作**
  - [ ] 实现 WebSocket 服务，用于实时推送工作流执行状态
  - [ ] _（高级）_ 实现多人实时协作编辑同一个工作流的功能
