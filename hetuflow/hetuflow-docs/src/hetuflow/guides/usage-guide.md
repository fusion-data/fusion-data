# hetuflow 使用指南

## 概述

hetuflow 是一个现代化的分布式任务调度系统，基于 Rust 2024 Edition 构建，提供了高性能、高可靠性的任务调度能力。本指南将帮助您快速上手使用 hetuflow。

### 核心特性

- **三层任务模型**: Job → Task → TaskInstance 的清晰架构
- **WebSocket 全双工通信**: 支持实时任务分发和状态上报
- **类型安全**: 基于 Rust 类型系统的编译时检查
- **多种调度策略**: 支持 Cron、间隔、守护进程、事件驱动等
- **丰富的执行环境**: 支持 Bash、Python、Node.js、Java 等
- **资源限制**: 支持内存、CPU、执行时间等资源约束
- **标签匹配**: 灵活的任务-Agent 匹配机制

## 快速开始

### 环境准备

#### 系统要求

- **操作系统**: Linux、macOS、Windows
- **Rust 版本**: ≥ 1.89
- **Node.js 版本**: ≥ 22
- **Docker**: 用于数据库服务
- **PostgreSQL**: ≥ 13

#### 安装依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 安装 Node.js (使用 nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 22
nvm use 22

# 安装 pnpm
npm install -g pnpm
```

### 数据库服务

使用 Docker 启动 PostgreSQL 和 Redis：

```bash
# 启动服务
docker-compose up -d

# 检查服务状态
docker-compose ps
```

### 编译项目

```bash
# 克隆项目
git clone <repository-url>
cd hetuflow

# 编译整个 workspace
cargo build --workspace

# 运行测试
cargo test --workspace

# 格式化代码
cargo fmt
# 代码检查
cargo clippy --workspace --all-targets --all-features -- -D
```

## 基本概念

### 三层任务模型

hetuflow 采用三层任务模型来管理分布式任务执行：

```mermaid
graph TD
    subgraph "Job (作业定义)"
        J1[Job ID<br/>静态配置]
        J2[任务配置<br/>timeout, retries]
        J3[调度策略<br/>Cron, Interval]
        J4[环境变量<br/>PATH, PYTHONPATH]
    end

    subgraph "Task (任务计划)"
        T1[Task ID<br/>执行计划]
        T2[调度时间<br/>scheduled_at]
        T3[任务参数<br/>parameters]
        T4[优先级<br/>priority]
    end

    subgraph "TaskInstance (执行实例)"
        I1[Instance ID<br/>实际执行]
        I2[Agent ID<br/>执行节点]
        I3[执行状态<br/>status]
        I4[执行结果<br/>output, metrics]
    end

    J2 --> T2
    J3 --> T2
    T2 --> I2
    T3 --> I3
    T4 --> I4

    style J1 fill:#e3f2fd
    style T1 fill:#e8f5e8
    style I1 fill:#fff3e0
```

### 状态流转

任务状态在各个阶段之间流转：

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

## 启动服务

### 1. 启动 Server

```bash
# 开发模式启动
cargo run --bin hetuflow-server

# 生产模式启动
cargo run --release --bin hetuflow-server
```

Server 启动后会：

- 监听 HTTP 端口（默认 8080）
- 监听 WebSocket 端口（默认 8081）
- 连接 PostgreSQL 数据库
- 初始化数据库表结构
- 启动调度器和服务组件

### 2. 启动 Agent

```bash
# 开发模式启动
cargo run --bin hetuflow-agent

# 生产模式启动
cargo run --release --bin hetuflow-agent
```

Agent 启动后会：

- 连接到 Server 的 WebSocket 端口
- 发送注册请求
- 开始心跳维持
- 等待任务分发

### 3. 验证服务状态

```bash
# 检查 Server 健康状态
curl http://localhost:8080/api/v1/system/health

# 检查 API 文档
open http://localhost:8080/docs
```

## 创建第一个任务

### 1. 创建 Job

Job 定义了要执行的任务的静态配置：

```bash
curl -X POST http://localhost:8080/api/v1/jobs/item \
  -H "Content-Type: application/json" \
  -d '{
    "namespace_id": "default",
    "name": "hello-world-job",
    "description": "Hello World 示例任务",
    "environment": {
      "MESSAGE": "Hello from hetuflow!"
    },
    "config": {
      "timeout": 300,
      "max_retries": 2,
      "retry_interval": 30,
      "cmd": "bash",
      "args": ["-c", "echo $MESSAGE"],
      "capture_output": true,
      "max_output_size": 1024000
    }
  }'
```

### 2. 创建 Schedule

Schedule 定义了任务的调度策略：

```bash
curl -X POST http://localhost:8080/api/v1/schedules/item \
  -H "Content-Type: application/json" \
  -d '{
    "job_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "every-minute",
    "kind": "interval",
    "cron_expression": null,
    "interval_seconds": 60,
    "enabled": true,
    "start_time": null,
    "end_time": null
  }'
```

### 3. 监控任务执行

```bash
# 查看生成的 Tasks
curl -X POST http://localhost:8080/api/v1/tasks/page \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {
      "job_id": {"eq": "550e8400-e29b-41d4-a716-446655440000"}
    },
    "page": {"page": 1, "page_size": 10}
  }'

# 查看任务实例
curl -X POST http://localhost:8080/api/v1/task-instances/page \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {
      "job_id": {"eq": "550e8400-e29b-41d4-a716-446655440000"}
    },
    "page": {"page": 1, "page_size": 10}
  }'
```

## 高级配置

### 1. 多种执行命令

#### Python 脚本执行

```json
{
  "config": {
    "cmd": "python",
    "args": ["script.py", "--input", "data.csv"],
    "environment": {
      "PYTHONPATH": "/opt/app/scripts"
    }
  }
}
```

#### Node.js 脚本执行

```json
{
  "config": {
    "cmd": "node",
    "args": ["process.js"],
    "environment": {
      "NODE_ENV": "production"
    }
  }
}
```

#### Java 程序执行

```json
{
  "config": {
    "cmd": "java",
    "args": ["-jar", "app.jar", "--server.port=8080"],
    "environment": {
      "JAVA_OPTS": "-Xmx512m"
    }
  }
}
```

### 2. 资源限制

```json
{
  "config": {
    "timeout": 3600,
    "resource_limits": {
      "max_memory_mb": 1024,
      "max_cpu_percent": 0.8,
      "max_execution_time_secs": 1800,
      "max_output_size_bytes": 10485760
    }
  }
}
```

### 3. 标签匹配

为 Job 设置标签，只有匹配的 Agent 才会执行：

```json
{
  "config": {
    "labels": {
      "env": "production",
      "region": "east",
      "gpu": "true"
    }
  }
}
```

Agent 注册时声明自己的标签：

```bash
# Agent 配置文件 agent.toml
[labels]
env = "production"
region = "east"
gpu = "true"
```

### 4. 复杂调度策略

#### Cron 表达式

```json
{
  "kind": "cron",
  "cron_expression": "0 0 12 * * ?",  # 每天 12:00
  "timezone": "Asia/Shanghai"
}
```

#### 间隔调度

```json
{
  "kind": "interval",
  "interval_seconds": 300,  # 每 5 分钟
  "start_time": "2024-01-01T00:00:00Z"
}
```

#### 事件驱动

```json
{
  "kind": "event",
  "event_type": "file.created",
  "event_filter": {
    "path": "/data/input/*.csv"
  }
}
```

## 监控和管理

### 1. 服务监控

```bash
# 健康检查
curl http://localhost:8080/api/v1/system/health

# 系统指标
curl http://localhost:8080/api/v1/system/metrics

# Gateway 统计
curl http://localhost:8080/api/v1/gateway/stats
```

### 2. Agent 管理

```bash
# 查看 Agent 列表
curl -X POST http://localhost:8080/api/v1/agents/query \
  -H "Content-Type: application/json" \
  -d '{"page": {"page": 1, "page_size": 20}}'

# 查看 Agent 详情
curl http://localhost:8080/api/v1/agents/agent-01

# 更新 Agent 标签
curl -X POST http://localhost:8080/api/v1/agents/agent-01/update \
  -H "Content-Type: application/json" \
  -d '{
    "labels": {
      "env": "staging",
      "capacity": "high"
    }
  }'
```

### 3. 任务管理

```bash
# 查询运行中的任务
curl -X POST http://localhost:8080/api/v1/tasks/page \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {"status": {"eq": 10}},
    "page": {"page": 1, "page_size": 20}
  }'

# 重试失败的任务
curl -X POST http://localhost:8080/api/v1/tasks/550e8400-e29b-41d4-a716-446655440001/retry

# 取消任务
curl -X POST http://localhost:8080/api/v1/tasks/550e8400-e29b-41d4-a716-446655440001/cancel
```

## 故障排除

### 1. 常见问题

#### Agent 无法连接

检查以下几点：

1. Server 是否正常启动
2. WebSocket 端口是否可访问
3. Agent 配置是否正确
4. 网络连接是否正常

```bash
# 检查 Server 日志
cargo run --bin hetuflow-server 2>&1 | grep -i error

# 检查 Agent 日志
cargo run --bin hetuflow-agent 2>&1 | grep -i error

# 测试网络连接
telnet localhost 8081
```

#### 任务执行失败

查看任务实例的错误信息：

```bash
# 查询失败的任务实例
curl -X POST http://localhost:8080/api/v1/task-instances/page \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {"status": {"eq": 90}},
    "page": {"page": 1, "page_size": 10}
  }'
```

### 2. 日志配置

调整日志级别获取更详细的调试信息：

```toml
# 配置文件 config.toml
[logging]
level = "debug"  # debug, info, warn, error
format = "json"  # json, text
```

### 3. 数据库连接问题

检查数据库连接和表结构：

```bash
# 检查 PostgreSQL 连接
psql -h localhost -p 5432 -U postgres -d hetuflow

# 查看数据库表
\dt

# 检查表数据
SELECT * FROM sched_job LIMIT 10;
SELECT * FROM sched_agent LIMIT 10;
```

## 性能优化

### 1. 数据库优化

```sql
-- 创建索引
CREATE INDEX idx_sched_task_status ON sched_task(status);
CREATE INDEX idx_sched_task_scheduled_at ON sched_task(scheduled_at);
CREATE INDEX idx_sched_task_instance_status ON sched_task_instance(status);
CREATE INDEX idx_sched_task_instance_task_id ON sched_task_instance(task_id);

-- 分区表（对于大量数据）
CREATE TABLE sched_task_instance_2024_01 PARTITION OF sched_task_instance
  FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
```

### 2. 配置优化

```toml
# Server 配置优化
[server]
http_port = 8080
websocket_port = 8081
max_connections = 1000

[database]
pool_size = 20
max_lifetime = 3600  # 连接最大生命周期（秒）

[scheduler]
tick_interval = 1  # 调度器检查间隔（秒）
max_concurrent_tasks = 100  # 最大并发任务数
```

### 3. Agent 优化

```toml
# Agent 配置优化
[agent]
heartbeat_interval = 30  # 心跳间隔（秒）
task_check_interval = 10  # 任务检查间隔（秒）
max_concurrent_tasks = 5  # 最大并发任务数

[executor]
work_dir = "/tmp/hetuflow"
cleanup_enabled = true
cleanup_interval = 3600  # 清理间隔（秒）
```

## 部署指南

### 1. 生产环境部署

#### 使用 Docker

```dockerfile
# Dockerfile.server
FROM rust:1.89-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin hetuflow-server

FROM alpine:latest
COPY --from=builder /app/target/release/hetuflow-server /usr/local/bin/
EXPOSE 8080 8081
CMD ["hetuflow-server"]
```

```dockerfile
# Dockerfile.agent
FROM rust:1.89-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin hetuflow-agent

FROM alpine:latest
COPY --from=builder /app/target/release/hetuflow-agent /usr/local/bin/
CMD ["hetuflow-agent"]
```

#### 使用 Docker Compose

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: hetuflow
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  hetuflow-server:
    build:
      context: .
      dockerfile: Dockerfile.server
    ports:
      - "8080:8080"
      - "8081:8081"
    environment:
      - DATABASE_URL=postgres://postgres:password@postgres:5432/hetuflow
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis

  hetuflow-agent:
    build:
      context: .
      dockerfile: Dockerfile.agent
    environment:
      - SERVER_URL=ws://hetuflow-server:8081
    depends_on:
      - hetuflow-server
    deploy:
      replicas: 3

volumes:
  postgres_data:
```

### 2. Kubernetes 部署

```yaml
# hetuflow-server-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hetuflow-server
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hetuflow-server
  template:
    metadata:
      labels:
        app: hetuflow-server
    spec:
      containers:
      - name: hetuflow-server
        image: hetuflow/server:latest
        ports:
        - containerPort: 8080
        - containerPort: 8081
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: hetuflow-secret
              key: database-url
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
---
apiVersion: v1
kind: Service
metadata:
  name: hetuflow-server-service
spec:
  selector:
    app: hetuflow-server
  ports:
  - port: 8080
    targetPort: 8080
    name: http
  - port: 8081
    targetPort: 8081
    name: websocket
```

```yaml
# hetuflow-agent-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hetuflow-agent
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hetuflow-agent
  template:
    metadata:
      labels:
        app: hetuflow-agent
    spec:
      containers:
      - name: hetuflow-agent
        image: hetuflow/agent:latest
        env:
        - name: SERVER_URL
          value: "ws://hetuflow-server-service:8081"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

## 最佳实践

### 1. 任务设计原则

- **原子性**: 每个 Job 应该执行一个独立的任务
- **幂等性**: 任务可以安全地重复执行
- **超时设置**: 为所有任务设置合理的超时时间
- **错误处理**: 处理可能的异常情况
- **资源控制**: 设置适当的资源限制

### 2. 监控和告警

- **健康检查**: 定期检查服务状态
- **性能监控**: 监控任务执行时间和资源使用
- **错误告警**: 及时发现和处理失败任务
- **容量规划**: 根据 Agent 负载调整任务分发

### 3. 安全考虑

- **认证授权**: 使用 JWT 进行 API 认证
- **网络安全**: 使用 TLS 加密通信
- **资源隔离**: 使用命名空间隔离不同环境的任务
- **访问控制**: 基于角色的访问控制（RBAC）

### 4. 数据管理

- **定期备份**: 备份重要的任务配置和执行记录
- **数据清理**: 定期清理过期的任务实例日志
- **性能优化**: 为大表创建合适的索引
- **容量规划**: 预估数据增长并提前规划

## 扩展开发

### 1. 自定义执行命令

扩展 `ExecuteCommand` 枚举：

```rust
// hetuflow-core/src/types/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr)]
pub enum ExecuteCommand {
  Bash,
  Python,
  Node,
  // 自定义命令
  Custom("custom"),
}
```

### 2. 自定义调度器

实现自定义调度逻辑：

```rust
// 自定义调度器
pub struct CustomScheduler {
  // 调度器状态
}

impl Scheduler for CustomScheduler {
  async fn schedule(&self, task: &SchedTask) -> Result<(), SchedulerError> {
    // 自定义调度逻辑
  }
}
```

### 3. 插件开发

使用 Extism 开发 WebAssembly 插件：

```rust
// 插件开发示例
#[no_mangle]
pub extern "C" fn execute_task() {
  // 插件逻辑
}
```

## 总结

hetuflow 提供了一个强大而灵活的分布式任务调度平台。通过本指南，您应该能够：

1. 理解 hetuflow 的核心概念和架构
2. 快速部署和运行 hetuflow 系统
3. 创建和管理各种类型的任务
4. 监控系统状态和任务执行
5. 进行故障排除和性能优化
6. 在生产环境中部署 hetuflow

随着使用的深入，您可以探索更多高级特性，如自定义调度器、插件开发、多租户支持等，充分发挥 hetuflow 的潜力。