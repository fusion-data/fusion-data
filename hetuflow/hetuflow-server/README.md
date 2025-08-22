# Hetuflow Server

一个高性能、可扩展的任务调度服务器，支持 WebSocket 连接、RESTful API 和 Cron 作业。

## 功能特性

- 🚀 **高性能调度引擎**：基于 Rust 和 Tokio 构建
- 🔄 **WebSocket 支持**：实时 Agent 连接和通信
- 📡 **RESTful API**：完整的 HTTP API 接口
- ⏰ **Cron 作业**：定时任务调度
- 🎯 **智能分发**：基于 Agent 能力的任务分发
- 📊 **实时监控**：任务和 Agent 状态监控
- 🔧 **CLI 工具**：命令行管理工具

## 快速开始

### 1. 环境准备

确保已安装：

- Rust 1.75+
- PostgreSQL 14+
- Docker（可选）

### 2. 配置数据库

```bash
# 创建数据库
createdb hetuflow

# 运行迁移
sqlx migrate run
```

### 3. 配置文件

复制配置文件模板：

```bash
cp config.example.toml config.toml
# 编辑 config.toml 配置数据库连接
```

### 4. 启动服务

```bash
# 启动服务器
cargo run -- start --bind 0.0.0.0:8080

# 或者使用CLI
./hetuflow-server start --bind 0.0.0.0:8080
```

## 使用方法

### CLI 命令

```bash
# 启动服务器
./hetuflow-server start

# 创建任务
./hetuflow-server create-task \
  --name "test-task" \
  --task-type "data-processing" \
  --priority high \
  --payload '{"data": "test"}'

# 列出任务
./hetuflow-server list-tasks

# 创建Cron作业
./hetuflow-server create-cron-job \
  --name "daily-backup" \
  --cron-expression "0 2 * * *" \
  --task-type "backup" \
  --payload '{"type": "full"}'

# 列出Agent
./hetuflow-server list-agents --online

# 调度任务
./hetuflow-server dispatch-task \
  --task-type "web-scraping" \
  --payload '{"url": "https://example.com"}' \
  --capabilities chrome headless
```

### RESTful API

#### 任务管理

```bash
# 创建任务
curl -X POST http://localhost:8080/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-task",
    "task_type": "data-processing",
    "priority": "high",
    "payload": {"data": "test"}
  }'

# 获取任务列表
curl http://localhost:8080/api/tasks

# 获取单个任务
curl http://localhost:8080/api/tasks/{task_id}

# 取消任务
curl -X DELETE http://localhost:8080/api/tasks/{task_id}
```

#### Agent 管理

```bash
# 获取Agent列表
curl http://localhost:8080/api/agents

# 创建Agent
curl -X POST http://localhost:8080/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "agent-1",
    "capabilities": ["chrome", "headless"],
    "status": "active"
  }'
```

#### Cron 作业

```bash
# 创建Cron作业
curl -X POST http://localhost:8080/api/cron-jobs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "daily-backup",
    "cron_expression": "0 2 * * *",
    "task_type": "backup",
    "payload": {"type": "full"},
    "enabled": true
  }'
```

### WebSocket 连接

Agent 可以通过 WebSocket 连接到服务器：

```javascript
const ws = new WebSocket("ws://localhost:8080/ws");

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log("Received:", message);
};

// 发送Agent能力
ws.send(
  JSON.stringify({
    type: "AgentCapabilities",
    capabilities: {
      capabilities: ["chrome", "headless"],
      max_concurrent_tasks: 3,
      supported_task_types: ["web-scraping", "data-processing"],
    },
  })
);

// 发送心跳
ws.send(
  JSON.stringify({
    type: "AgentHeartbeat",
    agent_id: "agent-1",
    timestamp: new Date().toISOString(),
  })
);

// 发送任务结果
ws.send(
  JSON.stringify({
    type: "TaskResult",
    task_id: "task-123",
    result: { status: "success", data: "scraped-data" },
    error: null,
  })
);
```

## 架构设计

### 核心组件

- **SchedulerEngine**: 任务调度引擎
- **TaskQueue**: 任务队列管理
- **TaskDispatcher**: 任务分发器
- **GatewaySvc**: WebSocket 网关服务
- **AgentSvc**: Agent 管理服务
- **CronService**: 定时任务服务

### 数据流

1. **任务创建**: 通过 API 或 CLI 创建任务
2. **任务调度**: SchedulerEngine 处理调度逻辑
3. **任务分发**: TaskDispatcher 分发给合适的 Agent
4. **任务执行**: Agent 通过 WebSocket 接收并执行任务
5. **结果收集**: Agent 返回结果，系统更新状态

## 配置说明

### 服务器配置

```toml
[server]
bind_address = "0.0.0.0:8080"  # 监听地址
workers = 4                      # 工作线程数

[database]
type = "postgres"                # 数据库类型
host = "localhost"              # 主机
port = 5432                      # 端口
database = "hetuflow"    # 数据库名
max_connections = 100            # 最大连接数
```

### 调度器配置

```toml
[scheduler]
max_concurrent_tasks = 100       # 最大并发任务数
retry_interval_seconds = 60      # 重试间隔（秒）
cleanup_interval_seconds = 3600  # 清理间隔（秒）
```

## 开发

### 运行测试

```bash
cargo test
```

### 代码格式化

```bash
cargo fmt
```

### 代码检查

```bash
cargo clippy
```

### 数据库迁移

```bash
# 创建新的迁移
sqlx migrate add create_new_table

# 运行迁移
sqlx migrate run

# 回滚迁移
sqlx migrate revert
```

## Docker 部署

```bash
# 构建镜像
docker build -t hetuflow-server .

# 运行容器
docker run -d \
  --name hetuflow \
  -p 8080:8080 \
  -e DATABASE_URL=postgresql://user:pass@host:5432/hetuflow \
  hetuflow-server
```

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 许可证

本项目采用 Apache-2.0 和 商业双重许可。详见 [LICENSE](../../LICENSE.txt) 和 [LICENSE-COMMERCIAL.txt](../../LICENSE-COMMERCIAL.txt) 文件。
