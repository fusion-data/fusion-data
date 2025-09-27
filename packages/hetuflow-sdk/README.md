# Hetuflow TypeScript SDK

基于 Hetuflow OpenAPI 规范生成的 TypeScript SDK，提供完整的类型安全的 API 访问。

## 安装

```bash
pnpm add @fusion-data/hetuflow-sdk
```

## 快速开始

### 基础用法

```typescript
import { HetuflowSDK } from "@fusion-data/hetuflow-sdk";

// 创建 SDK 实例
const sdk = new HetuflowSDK({
  baseURL: "http://localhost:9500",
  timeout: 30000,
});

// 可选：设置认证 token
sdk.setToken("your-jwt-token");
```

### Agent 管理

```typescript
// 创建 Agent
const agent = await sdk.agents.createAgent({
  name: "My Agent",
  description: "A sample agent",
  config: { key: "value" },
});

// 查询 Agent 列表
const agents = await sdk.agents.queryAgents({
  page: 1,
  limit: 10,
  name: "My",
});

// 获取单个 Agent
const singleAgent = await sdk.agents.getAgent(agent.id);

// 更新 Agent
await sdk.agents.updateAgent(agent.id, {
  name: "Updated Agent Name",
});

// 删除 Agent
await sdk.agents.deleteAgent(agent.id);
```

### Job 管理

```typescript
// 创建 Job
const job = await sdk.jobs.createJob({
  name: "Daily Task",
  description: "Runs daily at midnight",
  cron_expr: "0 0 * * *",
  agent_id: "agent-uuid",
  config: { param1: "value1" },
});

// 查询 Job 列表
const jobs = await sdk.jobs.queryJobs({
  page: 1,
  limit: 10,
  agent_id: "agent-uuid",
});

// 启用/禁用 Job
await sdk.jobs.enableJob(job.id);
await sdk.jobs.disableJob(job.id);
```

### Task 管理

```typescript
// 创建 Task
const task = await sdk.tasks.createTask({
  name: "Process Data",
  description: "Process incoming data",
  job_id: "job-uuid",
  config: { input: "data.csv" },
});

// 查询 Task 列表
const tasks = await sdk.tasks.queryTasks({
  page: 1,
  limit: 10,
  job_id: "job-uuid",
});

// 取消/重试 Task
await sdk.tasks.cancelTask(task.id);
await sdk.tasks.retryTask(task.id);
```

### TaskInstance 管理

```typescript
// 创建 TaskInstance
const instanceId = await sdk.taskInstances.createTaskInstance({
  task_id: "task-uuid",
  config: { runtime_param: "value" },
});

// 查询 TaskInstance 列表
const instances = await sdk.taskInstances.queryTaskInstances({
  page: 1,
  limit: 10,
  task_id: "task-uuid",
  status: "running",
});
```

### 认证

```typescript
// 生成 Token
const tokenResponse = await sdk.auth.generateToken({
  subject: "user@example.com",
  expires_in: 3600, // 1 hour
});

// 设置 Token
sdk.setToken(tokenResponse.token);
```

### 网关操作

```typescript
// 发送命令
const commandResult = await sdk.gateway.sendCommand({
  agent_id: "agent-uuid",
  command: "execute",
  args: { param: "value" },
});

// 获取网关状态
const status = await sdk.gateway.getStatus();

// WebSocket 连接
const wsUrl = sdk.gateway.getWebSocketUrl("agent-uuid");
const ws = new WebSocket(wsUrl);
```

### 系统信息

```typescript
// 健康检查
const health = await sdk.system.getHealth();
console.log("System status:", health.status);

// 获取系统信息
const info = await sdk.system.getInfo();

// 获取指标
const metrics = await sdk.system.getMetrics();
```

## 错误处理

```typescript
import { HetuflowError } from "@fusion-data/hetuflow-sdk";

try {
  const agent = await sdk.agents.getAgent("invalid-id");
} catch (error) {
  if (error instanceof HetuflowError) {
    console.error("API Error:", {
      message: error.message,
      status: error.status,
      code: error.code,
      details: error.details,
    });
  } else {
    console.error("Unexpected error:", error);
  }
}
```

## TypeScript 支持

SDK 提供完整的 TypeScript 类型定义：

```typescript
import type { SchedAgent, SchedJob, TaskStatus, PageResult } from "@fusion-data/hetuflow-sdk";

const agent: SchedAgent = await sdk.agents.getAgent("id");
const jobs: PageResult<SchedJob> = await sdk.jobs.queryJobs({});
```

## 配置选项

```typescript
const sdk = new HetuflowSDK({
  baseURL: "http://localhost:9500", // 必需
  timeout: 30000, // 请求超时时间（毫秒）
  headers: {
    // 自定义请求头
    "X-Custom-Header": "value",
  },
  token: "jwt-token", // 可选的初始认证 token
});
```

## API 参考

详细的 API 文档请参考 [Hetuflow OpenAPI 文档](http://localhost:9500/docs/)。

## 许可证

Apache-2.0
