# hetuflow Server API 文档

## 概述

hetuflow Server 提供了一套完整的 RESTful API，用于管理分布式任务调度系统。所有 API 基于 Axum 框架实现，支持 OpenAPI 文档生成，使用 `/api/v1` 路径前缀。

### 技术特性

- **OpenAPI 集成**: 使用 `utoipa` 和 `utoipa-axum` 自动生成 API 文档
- **类型安全**: 基于 Rust 类型系统的编译时检查
- **统一响应**: 使用 `WebResult` 统一错误处理
- **分页查询**: 内置分页和过滤支持
- **服务层架构**: 通过 Service 层提供业务逻辑封装

### API 结构

```rust
// API 路由结构
/api/v1/
├── agents          # Agent 管理
├── servers         # 服务器管理
├── jobs           # 作业管理
├── schedules      # 调度管理
├── tasks          # 任务管理
├── task-instances # 任务实例管理
├── system         # 系统监控
├── gateway        # 网关管理
└── auth           # 认证授权
```

## Agent 管理 API

### 基础信息

- **路径**: `/api/v1/agents`
- **模块**: [`agents.rs`](../../../hetuflow-server/src/endpoint/api/v1/agents.rs)
- **服务**: [`AgentSvc`](../../../hetuflow-server/src/service/agent_svc.rs)

### API 端点

#### 1. 查询 Agent 列表

**端点**: `POST /api/v1/agents/query`

**请求**:

```json
{
  "filter": {
    "id": { "eq": "550e8400-e29b-41d4-a716-446655440000" },
    "namespace_id": { "eq": "default" },
    "created_at": { "gte": "2024-01-01T00:00:00Z" }
  },
  "page": {
    "page": 1,
    "page_size": 20
  }
}
```

**响应**:

```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "namespace_id": "default",
      "name": "agent-01",
      "endpoint": "ws://localhost:8081",
      "status": "online",
      "labels": { "env": "production", "region": "east" },
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

**Rust 代码示例**:

```rust
// 请求类型
use hetuflow_core::models::{AgentForQuery, SchedAgent};
use fusionsql::page::PageResult;

// 服务调用
let result = agent_svc.query(query_request).await?;
```

#### 2. 创建 Agent

**端点**: `POST /api/v1/agents/create`

**请求**:

```json
{
  "namespace_id": "default",
  "name": "agent-01",
  "endpoint": "ws://localhost:8081",
  "labels": { "env": "production", "region": "east" }
}
```

**响应**:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### 3. 获取 Agent 详情

**端点**: `GET /api/v1/agents/{id}`

**响应**:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "namespace_id": "default",
  "name": "agent-01",
  "endpoint": "ws://localhost:8081",
  "status": "online",
  "labels": { "env": "production", "region": "east" },
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### 4. 更新 Agent

**端点**: `POST /api/v1/agents/{id}/update`

**请求**:

```json
{
  "name": "agent-01-updated",
  "labels": { "env": "staging", "region": "west" }
}
```

#### 5. 删除 Agent

**端点**: `DELETE /api/v1/agents/{id}`

## Job 管理 API

### 基础信息

- **路径**: `/api/v1/jobs`
- **模块**: [`jobs.rs`](../../../hetuflow-server/src/endpoint/api/v1/jobs.rs)
- **服务**: [`JobSvc`](../../../hetuflow-server/src/service/job_svc.rs)

### API 端点

#### 1. 查询 Job 列表

**端点**: `POST /api/v1/jobs/page`

**请求**:

```json
{
  "filter": {
    "namespace_id": { "eq": "default" },
    "status": { "eq": 1 },
    "created_at": { "gte": "2024-01-01T00:00:00Z" }
  },
  "page": {
    "page": 1,
    "page_size": 20
  }
}
```

**响应**:

```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "namespace_id": "default",
      "name": "data-processing-job",
      "description": "数据处理作业",
      "environment": { "PYTHONPATH": "/opt/app" },
      "config": {
        "timeout": 3600,
        "max_retries": 3,
        "retry_interval": 60,
        "cmd": "python",
        "args": ["process.py"],
        "capture_output": true,
        "max_output_size": 10485760
      },
      "status": "enabled",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

#### 2. 创建 Job

**端点**: `POST /api/v1/jobs/item`

**请求**:

```json
{
  "namespace_id": "default",
  "name": "data-processing-job",
  "description": "数据处理作业",
  "environment": { "PYTHONPATH": "/opt/app" },
  "config": {
    "timeout": 3600,
    "max_retries": 3,
    "retry_interval": 60,
    "cmd": "python",
    "args": ["process.py"],
    "capture_output": true,
    "max_output_size": 10485760
  }
}
```

**响应**:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### 3. 获取 Job 详情

**端点**: `GET /api/v1/jobs/item/{id}`

#### 4. 更新 Job

**端点**: `PUT /api/v1/jobs/item/{id}`

#### 5. 删除 Job

**端点**: `DELETE /api/v1/jobs/item/{id}`

#### 6. 启用 Job

**端点**: `POST /api/v1/jobs/item/{id}/enable`

#### 7. 禁用 Job

**端点**: `POST /api/v1/jobs/item/{id}/disable`

## Task 管理 API

### 基础信息

- **路径**: `/api/v1/tasks`
- **模块**: [`tasks.rs`](../../../hetuflow-server/src/endpoint/api/v1/tasks.rs)
- **服务**: [`TaskSvc`](../../../hetuflow-server/src/service/task_svc.rs)

### API 端点

#### 1. 查询 Task 列表

**端点**: `POST /api/v1/tasks/page`

**请求**:

```json
{
  "filter": {
    "job_id": { "eq": "550e8400-e29b-41d4-a716-446655440000" },
    "status": { "eq": 1 },
    "scheduled_at": { "gte": "2024-01-01T00:00:00Z" }
  },
  "page": {
    "page": 1,
    "page_size": 20
  }
}
```

**响应**:

```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "job_id": "550e8400-e29b-41d4-a716-446655440000",
      "namespace_id": "default",
      "priority": 0,
      "status": "pending",
      "schedule_id": "550e8400-e29b-41d4-a716-446655440002",
      "scheduled_at": "2024-01-01T10:00:00Z",
      "schedule_kind": "cron",
      "completed_at": null,
      "environment": { "PYTHONPATH": "/opt/app" },
      "parameters": { "input_file": "data.csv" },
      "config": {
        "timeout": 3600,
        "max_retries": 3,
        "retry_interval": 60,
        "cmd": "python",
        "args": ["process.py"],
        "capture_output": true,
        "max_output_size": 10485760
      },
      "retry_count": 0,
      "dependencies": null,
      "locked_at": null,
      "lock_version": 0,
      "created_by": 1,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_by": null,
      "updated_at": null
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

## TaskInstance 管理 API

### 基础信息

- **路径**: `/api/v1/task-instances`
- **模块**: [`task_instances.rs`](../../../hetuflow-server/src/endpoint/api/v1/task_instances.rs)
- **服务**: [`TaskInstanceSvc`](../../../hetuflow-server/src/service/task_svc.rs) # Note: Using TaskSvc for TaskInstance operations

### API 端点

#### 1. 查询 TaskInstance 列表

**端点**: `POST /api/v1/task-instances/page`

**请求**:

```json
{
  "filter": {
    "task_id": { "eq": "550e8400-e29b-41d4-a716-446655440001" },
    "agent_id": { "eq": "agent-01" },
    "status": { "eq": 10 }
  },
  "page": {
    "page": 1,
    "page_size": 20
  }
}
```

**响应**:

```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440003",
      "task_id": "550e8400-e29b-41d4-a716-446655440001",
      "job_id": "550e8400-e29b-41d4-a716-446655440000",
      "agent_id": "agent-01",
      "status": "running",
      "started_at": "2024-01-01T10:00:00Z",
      "completed_at": null,
      "output": null,
      "error_message": null,
      "exit_code": null,
      "metrics": null,
      "created_at": "2024-01-01T10:00:00Z",
      "updated_at": "2024-01-01T10:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

## Schedule 管理 API

### 基础信息

- **路径**: `/api/v1/schedules`
- **模块**: [`schedules.rs`](../../../hetuflow-server/src/endpoint/api/v1/schedules.rs)
- **服务**: [`ScheduleSvc`](../../../hetuflow-server/src/service/schedule_svc.rs)

### API 端点

#### 1. 查询 Schedule 列表

**端点**: `POST /api/v1/schedules/page`

**请求**:

```json
{
  "filter": {
    "job_id": { "eq": "550e8400-e29b-41d4-a716-446655440000" },
    "kind": { "eq": 1 },
    "enabled": { "eq": true }
  },
  "page": {
    "page": 1,
    "page_size": 20
  }
}
```

## System 管理 API

### 基础信息

- **路径**: `/api/v1/system`
- **模块**: [`system.rs`](../../../hetuflow-server/src/endpoint/api/v1/system.rs)
- **服务**: [`SystemSvc`](../../../hetuflow-server/src/service/server_svc.rs) # Note: System functionality handled by ServerSvc

### API 端点

#### 1. 健康检查

**端点**: `GET /api/v1/system/health`

**响应**:

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "version": "0.1.1",
  "uptime": 3600,
  "database": {
    "status": "connected",
    "pool_size": 10
  },
  "agents": {
    "total": 5,
    "online": 3,
    "offline": 2
  }
}
```

## Gateway 管理 API

### 基础信息

- **路径**: `/api/v1/gateway`
- **模块**: [`gateway.rs`](../../../hetuflow-server/src/endpoint/api/v1/gateway.rs)
- **服务**: [`GatewaySvc`](../../../hetuflow-server/src/application.rs) # Note: Gateway functionality integrated into ServerApplication

### API 端点

#### 1. 连接统计

**端点**: `GET /api/v1/gateway/stats`

**响应**:

```json
{
  "total_connections": 5,
  "active_connections": 3,
  "total_messages_sent": 1000,
  "total_messages_received": 950,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## 数据模型

### Agent 模型

```rust
// hetuflow-core/src/models/agent.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedAgent {
  pub id: String,                    // Agent ID
  pub namespace_id: String,          // 命名空间
  pub name: String,                  // Agent 名称
  pub endpoint: String,              // WebSocket 端点
  pub status: AgentStatus,           // Agent 状态
  pub labels: Option<Labels>,        // 标签
  pub created_at: DateTime<FixedOffset>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}
```

### Job 模型

```rust
// hetuflow-core/src/models/job.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedJob {
  pub id: Uuid,
  pub namespace_id: String,
  pub name: String,
  pub description: Option<String>,
  pub environment: Option<serde_json::Value>,
  pub config: TaskConfig,
  pub status: JobStatus,
  pub created_at: DateTime<FixedOffset>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}
```

### Task 模型

```rust
// hetuflow-core/src/models/task.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
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
  pub dependencies: Option<serde_json::Value>,
  pub locked_at: Option<DateTime<FixedOffset>>,
  pub lock_version: i32,
  pub created_by: i64,
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}
```

### TaskInstance 模型

```rust
// hetuflow-core/src/models/task_instance.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedTaskInstance {
  pub id: Uuid,
  pub task_id: Uuid,
  pub job_id: Uuid,
  pub agent_id: Option<String>,
  pub status: TaskInstanceStatus,
  pub started_at: Option<DateTime<FixedOffset>>,
  pub completed_at: Option<DateTime<FixedOffset>>,
  pub output: Option<String>,
  pub error_message: Option<String>,
  pub exit_code: Option<i32>,
  pub metrics: Option<TaskMetrics>,
  pub created_at: DateTime<FixedOffset>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}
```

## 状态枚举

### AgentStatus

```rust
pub enum AgentStatus {
  Offline = 0,    // 离线
  Online = 1,     // 在线
  Busy = 2,       // 忙碌
  Error = 3,      // 错误
}
```

### JobStatus

```rust
pub enum JobStatus {
  Disabled = 0,   // 禁用
  Enabled = 1,    // 启用
  Deleted = 2,    // 已删除
}
```

### TaskStatus

```rust
pub enum TaskStatus {
  Pending = 1,     // 等待分发
  Locked = 2,     // 已锁定
  Dispatched = 3, // 已分发
  Running = 10,    // 运行中
  Failed = 90,     // 错误
  Cancelled = 99,  // 取消完成
  Succeeded = 100, // 成功完成
}
```

### TaskInstanceStatus

```rust
pub enum TaskInstanceStatus {
  Pending = 1,     // 等待执行
  Dispatched = 5,  // 已分发
  Running = 10,    // 执行中
  Timeout = 20,    // 执行超时
  Failed = 90,     // 执行失败
  Succeeded = 100, // 执行成功
}
```

## 错误处理

### 统一错误格式

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Agent not found",
    "details": "Agent '123' does not exist"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 常见错误码

- `NOT_FOUND`: 资源不存在
- `VALIDATION_ERROR`: 参数验证失败
- `INTERNAL_ERROR`: 内部服务器错误
- `UNAUTHORIZED`: 未授权访问
- `FORBIDDEN`: 禁止访问

## 分页和过滤

### 分页参数

```json
{
  "page": {
    "page": 1, // 页码（从 1 开始）
    "page_size": 20 // 每页大小
  }
}
```

### 过滤参数

```json
{
  "filter": {
    "id": { "eq": "550e8400-e29b-41d4-a716-446655440000" }, // 等于
    "created_at": {
      "gte": "2024-01-01T00:00:00Z", // 大于等于
      "lte": "2024-12-31T23:59:59Z" // 小于等于
    },
    "status": { "in": [1, 2, 3] } // 包含
  }
}
```

## 认证和授权

### JWT 认证

所有 API 请求都需要在 Header 中包含有效的 JWT token：

```
Authorization: Bearer <jwt-token>
```

### 权限控制

- **读取权限**: 查询资源
- **写入权限**: 创建、更新、删除资源
- **管理权限**: 系统配置和监控

## 使用示例

### cURL 示例

```bash
# 创建 Job
curl -X POST http://localhost:8080/api/v1/jobs/item \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "namespace_id": "default",
    "name": "test-job",
    "description": "测试作业",
    "config": {
      "timeout": 3600,
      "max_retries": 3,
      "retry_interval": 60,
      "cmd": "bash",
      "args": ["-c", "echo Hello World"],
      "capture_output": true
    }
  }'

# 查询 Jobs
curl -X POST http://localhost:8080/api/v1/jobs/page \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {
      "namespace_id": { "eq": "default" }
    },
    "page": {
      "page": 1,
      "page_size": 10
    }
  }'
```

### Python 示例

```python
import requests
import json

# 配置
BASE_URL = "http://localhost:8080/api/v1"
TOKEN = "your-jwt-token"
HEADERS = {
    "Authorization": f"Bearer {TOKEN}",
    "Content-Type": "application/json"
}

# 创建 Job
def create_job():
    data = {
        "namespace_id": "default",
        "name": "python-job",
        "description": "Python 脚本执行",
        "config": {
            "timeout": 1800,
            "max_retries": 2,
            "retry_interval": 30,
            "cmd": "python",
            "args": ["script.py"],
            "capture_output": True
        }
    }

    response = requests.post(
        f"{BASE_URL}/jobs/item",
        headers=HEADERS,
        json=data
    )

    if response.status_code == 200:
        return response.json()
    else:
        raise Exception(f"API Error: {response.text}")

# 查询 Jobs
def query_jobs():
    data = {
        "filter": {
            "namespace_id": {"eq": "default"}
        },
        "page": {
            "page": 1,
            "page_size": 10
        }
    }

    response = requests.post(
        f"{BASE_URL}/jobs/page",
        headers=HEADERS,
        json=data
    )

    if response.status_code == 200:
        return response.json()
    else:
        raise Exception(f"API Error: {response.text}")
```

## 性能优化

### 批量操作

- 使用分页查询避免大量数据传输
- 合理设置 `page_size` 参数
- 使用过滤条件精确查询

### 缓存策略

- 静态数据缓存
- 查询结果缓存
- 连接池管理

### 连接管理

- 保持 HTTP 连接复用
- 使用连接池
- 合理设置超时时间

## 监控和日志

### API 监控

- 请求响应时间
- 错误率统计
- 并发连接数
- 资源使用情况

### 日志记录

- 请求日志
- 错误日志
- 性能日志
- 安全日志

## 版本兼容性

### API 版本

- 当前版本: `v1`
- 向后兼容: 保证
- 废弃通知: 提前 30 天

### 数据格式

- JSON 标准
- UTF-8 编码
- ISO 8601 时间格式
