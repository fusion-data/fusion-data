# Hetumind Web API 文档

本文档描述了 Hetumind Web 平台的 API 接口，包括工作流管理、节点执行、监控等功能。

## 📋 目录

- [基础信息](#基础信息)
- [认证方式](#认证方式)
- [工作流 API](#工作流-api)
- [节点 API](#节点-api)
- [执行 API](#执行-api)
- [监控 API](#监控-api)
- [配置 API](#配置-api)
- [错误处理](#错误处理)
- [数据模型](#数据模型)

## 基础信息

### 服务端点

- **开发环境**: `http://localhost:3001/api/v1`
- **测试环境**: `https://test-api.hetumind.com/api/v1`
- **生产环境**: `https://api.hetumind.com/api/v1`

### 协议支持

- **HTTP/HTTPS**: RESTful API
- **WebSocket**: 实时数据推送

### 内容类型

```http
Content-Type: application/json
Accept: application/json
```

## 认证方式

### Bearer Token 认证

```http
Authorization: Bearer <your-token>
```

### 获取访问令牌

#### 请求

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "your-password"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expiresIn": 3600,
    "user": {
      "id": "user-123",
      "email": "user@example.com",
      "name": "User Name",
      "role": "user"
    }
  }
}
```

### 刷新令牌

#### 请求

```http
POST /api/v1/auth/refresh
Content-Type: application/json

{
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expiresIn": 3600
  }
}
```

## 工作流 API

### 获取工作流列表

#### 请求

```http
GET /api/v1/workflows?page=1&limit=20&search=keyword&status=active
Authorization: Bearer <token>
```

#### 查询参数

| 参数      | 类型   | 必填 | 说明                |
| --------- | ------ | ---- | ------------------- |
| page      | number | 否   | 页码，默认 1        |
| limit     | number | 否   | 每页数量，默认 20   |
| search    | string | 否   | 搜索关键词          |
| status    | string | 否   | 工作流状态          |
| sortBy    | string | 否   | 排序字段            |
| sortOrder | string | 否   | 排序方向 (asc/desc) |

#### 响应

```json
{
  "success": true,
  "data": {
    "workflows": [
      {
        "id": "workflow-123",
        "name": "数据处理工作流",
        "description": "自动数据处理和清洗",
        "status": "active",
        "version": "1.2.0",
        "createdAt": "2024-01-15T10:30:00Z",
        "updatedAt": "2024-01-20T15:45:00Z",
        "createdBy": "user-123",
        "tags": ["data", "automation"],
        "metrics": {
          "totalExecutions": 1250,
          "successRate": 95.2,
          "avgExecutionTime": 45.6
        }
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 150,
      "totalPages": 8
    }
  }
}
```

### 获取工作流详情

#### 请求

```http
GET /api/v1/workflows/{workflowId}
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "workflow-123",
    "name": "数据处理工作流",
    "description": "自动数据处理和清洗",
    "status": "active",
    "version": "1.2.0",
    "nodes": [
      {
        "id": "node-1",
        "type": "trigger",
        "position": { "x": 100, "y": 100 },
        "data": {
          "label": "定时触发",
          "triggerType": "scheduled",
          "config": {
            "cronExpression": "0 9 * * 1-5"
          }
        }
      }
    ],
    "edges": [
      {
        "id": "edge-1",
        "source": "node-1",
        "target": "node-2",
        "sourceHandle": "output",
        "targetHandle": "input"
      }
    ],
    "variables": {
      "apiUrl": "https://api.example.com",
      "batchSize": 100
    },
    "settings": {
      "timeout": 300000,
      "retryAttempts": 3,
      "enableLogging": true
    },
    "createdAt": "2024-01-15T10:30:00Z",
    "updatedAt": "2024-01-20T15:45:00Z",
    "createdBy": "user-123"
  }
}
```

### 创建工作流

#### 请求

```http
POST /api/v1/workflows
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "新工作流",
  "description": "工作流描述",
  "nodes": [],
  "edges": [],
  "variables": {},
  "settings": {
    "timeout": 300000,
    "retryAttempts": 3
  },
  "tags": ["automation", "data"]
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "workflow-456",
    "name": "新工作流",
    "status": "draft",
    "version": "1.0.0",
    "createdAt": "2024-01-25T10:00:00Z",
    "createdBy": "user-123"
  }
}
```

### 更新工作流

#### 请求

```http
PUT /api/v1/workflows/{workflowId}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "更新后的工作流",
  "description": "更新后的描述",
  "nodes": [...],
  "edges": [...],
  "variables": {...}
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "workflow-123",
    "version": "1.3.0",
    "updatedAt": "2024-01-25T11:00:00Z",
    "updatedBy": "user-123"
  }
}
```

### 删除工作流

#### 请求

```http
DELETE /api/v1/workflows/{workflowId}
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "message": "工作流已删除"
}
```

### 复制工作流

#### 请求

```http
POST /api/v1/workflows/{workflowId}/clone
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "复制的工作流",
  "description": "基于原工作流创建的副本"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "workflow-789",
    "name": "复制的工作流",
    "status": "draft",
    "version": "1.0.0",
    "createdAt": "2024-01-25T12:00:00Z"
  }
}
```

## 节点 API

### 获取节点类型列表

#### 请求

```http
GET /api/v1/nodes/types
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": [
    {
      "type": "trigger",
      "name": "触发器",
      "description": "工作流触发节点",
      "category": "trigger",
      "icon": "thunderbolt",
      "configSchema": {
        "type": "object",
        "properties": {
          "triggerType": {
            "type": "string",
            "enum": ["manual", "scheduled", "webhook"]
          }
        }
      }
    },
    {
      "type": "aiAgent",
      "name": "AI Agent",
      "description": "AI 智能处理节点",
      "category": "ai",
      "icon": "robot",
      "configSchema": {
        "type": "object",
        "properties": {
          "agentType": {
            "type": "string",
            "enum": ["chat", "completion", "embedding"]
          },
          "model": {
            "type": "string",
            "enum": ["gpt-3.5-turbo", "gpt-4", "claude-3"]
          }
        }
      }
    }
  ]
}
```

### 获取节点配置模板

#### 请求

```http
GET /api/v1/nodes/templates/{nodeType}
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "type": "aiAgent",
    "defaultConfig": {
      "agentType": "chat",
      "model": "gpt-3.5-turbo",
      "temperature": 0.7,
      "maxTokens": 1024,
      "systemPrompt": "You are a helpful assistant."
    },
    "inputSchema": {
      "type": "object",
      "properties": {
        "message": {
          "type": "string",
          "description": "输入消息"
        }
      }
    },
    "outputSchema": {
      "type": "object",
      "properties": {
        "response": {
          "type": "string",
          "description": "AI 响应"
        },
        "usage": {
          "type": "object",
          "properties": {
            "promptTokens": { "type": "number" },
            "completionTokens": { "type": "number" },
            "totalTokens": { "type": "number" }
          }
        }
      }
    }
  }
}
```

### 验证节点配置

#### 请求

```http
POST /api/v1/nodes/validate
Authorization: Bearer <token>
Content-Type: application/json

{
  "type": "aiAgent",
  "config": {
    "agentType": "chat",
    "model": "gpt-3.5-turbo",
    "temperature": 0.7,
    "maxTokens": 1024
  }
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "valid": true,
    "errors": [],
    "warnings": [
      {
        "field": "temperature",
        "message": "温度值较高，可能影响输出稳定性"
      }
    ]
  }
}
```

## 执行 API

### 执行工作流

#### 请求

```http
POST /api/v1/workflows/{workflowId}/execute
Authorization: Bearer <token>
Content-Type: application/json

{
  "variables": {
    "inputData": "test data",
    "batchSize": 100
  },
  "options": {
    "timeout": 300000,
    "enableLogging": true
  }
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "executionId": "exec-123",
    "workflowId": "workflow-123",
    "status": "running",
    "startedAt": "2024-01-25T14:00:00Z",
    "estimatedDuration": 60000,
    "nodes": [
      {
        "id": "node-1",
        "status": "completed",
        "startedAt": "2024-01-25T14:00:00Z",
        "completedAt": "2024-01-25T14:00:05Z",
        "output": {
          "result": "node output data"
        }
      },
      {
        "id": "node-2",
        "status": "running",
        "startedAt": "2024-01-25T14:00:05Z"
      }
    ]
  }
}
```

### 获取执行状态

#### 请求

```http
GET /api/v1/executions/{executionId}
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "executionId": "exec-123",
    "workflowId": "workflow-123",
    "status": "completed",
    "startedAt": "2024-01-25T14:00:00Z",
    "completedAt": "2024-01-25T14:01:30Z",
    "duration": 90000,
    "nodes": [
      {
        "id": "node-1",
        "status": "completed",
        "startedAt": "2024-01-25T14:00:00Z",
        "completedAt": "2024-01-25T14:00:05Z",
        "duration": 5000,
        "output": {
          "result": "node output data"
        },
        "metrics": {
          "executionTime": 5000,
          "memoryUsage": 1024000,
          "cpuUsage": 15.2
        }
      }
    ],
    "variables": {
      "inputData": "test data",
      "processedData": "processed result"
    },
    "logs": [
      {
        "timestamp": "2024-01-25T14:00:00Z",
        "level": "info",
        "nodeId": "node-1",
        "message": "开始执行节点"
      },
      {
        "timestamp": "2024-01-25T14:00:05Z",
        "level": "info",
        "nodeId": "node-1",
        "message": "节点执行完成"
      }
    ],
    "metrics": {
      "totalDuration": 90000,
      "totalNodes": 5,
      "completedNodes": 5,
      "failedNodes": 0,
      "avgNodeDuration": 18000,
      "maxMemoryUsage": 5120000,
      "totalCost": 0.025
    }
  }
}
```

### 暂停执行

#### 请求

```http
POST /api/v1/executions/{executionId}/pause
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "executionId": "exec-123",
    "status": "paused",
    "pausedAt": "2024-01-25T14:00:30Z"
  }
}
```

### 恢复执行

#### 请求

```http
POST /api/v1/executions/{executionId}/resume
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "executionId": "exec-123",
    "status": "running",
    "resumedAt": "2024-01-25T14:01:00Z"
  }
}
```

### 取消执行

#### 请求

```http
POST /api/v1/executions/{executionId}/cancel
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "executionId": "exec-123",
    "status": "cancelled",
    "cancelledAt": "2024-01-25T14:00:45Z",
    "reason": "用户取消"
  }
}
```

### 获取执行历史

#### 请求

```http
GET /api/v1/workflows/{workflowId}/executions?page=1&limit=20&status=completed
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "executions": [
      {
        "executionId": "exec-123",
        "status": "completed",
        "startedAt": "2024-01-25T14:00:00Z",
        "completedAt": "2024-01-25T14:01:30Z",
        "duration": 90000,
        "successRate": 100,
        "triggeredBy": "user-123",
        "cost": 0.025
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 500,
      "totalPages": 25
    }
  }
}
```

## 监控 API

### 获取系统指标

#### 请求

```http
GET /api/v1/monitoring/metrics?timeRange=1h&granularity=5m
Authorization: Bearer <token>
```

#### 查询参数

| 参数        | 类型   | 必填 | 说明                       |
| ----------- | ------ | ---- | -------------------------- |
| timeRange   | string | 否   | 时间范围 (1h, 6h, 24h, 7d) |
| granularity | string | 否   | 数据粒度 (1m, 5m, 15m, 1h) |
| metrics     | string | 否   | 指定指标类型               |

#### 响应

```json
{
  "success": true,
  "data": {
    "timeRange": "1h",
    "granularity": "5m",
    "metrics": [
      {
        "timestamp": "2024-01-25T14:00:00Z",
        "workflowMetrics": {
          "totalWorkflows": 25,
          "activeWorkflows": 3,
          "completedWorkflows": 18,
          "failedWorkflows": 2,
          "avgExecutionTime": 45.6,
          "successRate": 92.5
        },
        "systemMetrics": {
          "cpuUsage": 65.2,
          "memoryUsage": 78.5,
          "diskUsage": 45.1,
          "networkIn": 1024000,
          "networkOut": 512000,
          "activeConnections": 156
        },
        "performanceMetrics": {
          "responseTime": 125.5,
          "throughput": 1250,
          "errorRate": 2.1,
          "availability": 99.8
        }
      }
    ]
  }
}
```

### 获取工作流统计

#### 请求

```http
GET /api/v1/monitoring/workflows/{workflowId}/stats?period=7d
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "workflowId": "workflow-123",
    "period": "7d",
    "executions": {
      "total": 150,
      "successful": 142,
      "failed": 5,
      "cancelled": 3,
      "successRate": 94.7
    },
    "performance": {
      "avgExecutionTime": 45600,
      "minExecutionTime": 12000,
      "maxExecutionTime": 120000,
      "p95ExecutionTime": 98000
    },
    "costs": {
      "totalCost": 2.45,
      "avgCostPerExecution": 0.0163,
      "costTrend": "+5.2%"
    },
    "errors": [
      {
        "type": "timeout",
        "count": 3,
        "lastOccurred": "2024-01-24T16:30:00Z"
      },
      {
        "type": "api_error",
        "count": 2,
        "lastOccurred": "2024-01-23T11:15:00Z"
      }
    ]
  }
}
```

### 获取告警列表

#### 请求

```http
GET /api/v1/monitoring/alerts?status=active&severity=high&page=1&limit=20
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "alerts": [
      {
        "id": "alert-123",
        "type": "performance",
        "severity": "high",
        "status": "active",
        "title": "工作流执行时间过长",
        "message": "工作流 '数据处理' 执行时间超过阈值",
        "workflowId": "workflow-123",
        "executionId": "exec-456",
        "triggeredAt": "2024-01-25T14:30:00Z",
        "acknowledgedAt": null,
        "resolvedAt": null,
        "metadata": {
          "threshold": 60000,
          "actualValue": 90000,
          "percentage": 150
        }
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 25,
      "totalPages": 2
    }
  }
}
```

### 创建告警规则

#### 请求

```http
POST /api/v1/monitoring/alert-rules
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "工作流执行时间告警",
  "description": "当工作流执行时间超过阈值时触发告警",
  "type": "performance",
  "severity": "medium",
  "condition": {
    "metric": "execution_time",
    "operator": "greater_than",
    "threshold": 60000,
    "timeWindow": "5m"
  },
  "actions": [
    {
      "type": "email",
      "config": {
        "recipients": ["admin@example.com"],
        "template": "workflow_performance_alert"
      }
    },
    {
      "type": "webhook",
      "config": {
        "url": "https://hooks.slack.com/...",
        "method": "POST",
        "headers": {
          "Content-Type": "application/json"
        }
      }
    }
  ],
  "enabled": true
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "rule-123",
    "name": "工作流执行时间告警",
    "type": "performance",
    "severity": "medium",
    "enabled": true,
    "createdAt": "2024-01-25T15:00:00Z",
    "createdBy": "user-123"
  }
}
```

## 配置 API

### 获取用户配置

#### 请求

```http
GET /api/v1/config/user
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "preferences": {
      "theme": "light",
      "language": "zh-CN",
      "timezone": "Asia/Shanghai",
      "notifications": {
        "email": true,
        "browser": true,
        "workflow": true,
        "alerts": true
      }
    },
    "settings": {
      "defaultTimeout": 300000,
      "maxConcurrentWorkflows": 5,
      "enableAutoSave": true,
      "showAdvancedOptions": false
    },
    "apiKeys": [
      {
        "id": "key-123",
        "name": "Production API Key",
        "key": "ak_live_...masked...",
        "permissions": ["read", "write", "execute"],
        "createdAt": "2024-01-15T10:00:00Z",
        "lastUsedAt": "2024-01-25T14:30:00Z"
      }
    ]
  }
}
```

### 更新用户配置

#### 请求

```http
PUT /api/v1/config/user
Authorization: Bearer <token>
Content-Type: application/json

{
  "preferences": {
    "theme": "dark",
    "language": "en-US",
    "notifications": {
      "email": false,
      "browser": true
    }
  },
  "settings": {
    "defaultTimeout": 600000,
    "maxConcurrentWorkflows": 10
  }
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "updatedAt": "2024-01-25T16:00:00Z",
    "updatedFields": ["preferences.theme", "preferences.language", "settings.defaultTimeout"]
  }
}
```

### 创建 API 密钥

#### 请求

```http
POST /api/v1/config/api-keys
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "New API Key",
  "permissions": ["read", "execute"],
  "expiresAt": "2025-01-25T00:00:00Z"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "key-456",
    "name": "New API Key",
    "key": "ak_live_abcdef123456789",
    "permissions": ["read", "execute"],
    "createdAt": "2024-01-25T16:30:00Z",
    "expiresAt": "2025-01-25T00:00:00Z"
  }
}
```

### 删除 API 密钥

#### 请求

```http
DELETE /api/v1/config/api-keys/{keyId}
Authorization: Bearer <token>
```

#### 响应

```json
{
  "success": true,
  "message": "API 密钥已删除"
}
```

## 错误处理

### 错误响应格式

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "请求参数验证失败",
    "details": [
      {
        "field": "email",
        "message": "邮箱格式不正确"
      }
    ],
    "timestamp": "2024-01-25T14:30:00Z",
    "requestId": "req-123456"
  }
}
```

### 错误代码

| 错误代码            | HTTP 状态码 | 说明             |
| ------------------- | ----------- | ---------------- |
| VALIDATION_ERROR    | 400         | 请求参数验证失败 |
| UNAUTHORIZED        | 401         | 未授权访问       |
| FORBIDDEN           | 403         | 权限不足         |
| NOT_FOUND           | 404         | 资源不存在       |
| CONFLICT            | 409         | 资源冲突         |
| RATE_LIMIT_EXCEEDED | 429         | 请求频率超限     |
| INTERNAL_ERROR      | 500         | 服务器内部错误   |
| SERVICE_UNAVAILABLE | 503         | 服务不可用       |

### 重试策略

对于以下错误代码，建议实现重试机制：

- `RATE_LIMIT_EXCEEDED`: 使用指数退避策略
- `INTERNAL_ERROR`: 短暂重试（最多 3 次）
- `SERVICE_UNAVAILABLE`: 等待服务恢复后重试

## 数据模型

### Workflow 工作流模型

```typescript
interface Workflow {
  id: string;
  name: string;
  description?: string;
  status: 'draft' | 'active' | 'inactive' | 'archived';
  version: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  variables: Record<string, any>;
  settings: WorkflowSettings;
  tags: string[];
  createdAt: string;
  updatedAt: string;
  createdBy: string;
}
```

### WorkflowNode 工作流节点模型

```typescript
interface WorkflowNode {
  id: string;
  type: string;
  position: { x: number; y: number };
  data: {
    label: string;
    description?: string;
    config: Record<string, any>;
    [key: string]: any;
  };
  inputs: string[];
  outputs: string[];
}
```

### Execution 执行模型

```typescript
interface Execution {
  executionId: string;
  workflowId: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled' | 'paused';
  startedAt: string;
  completedAt?: string;
  duration?: number;
  nodes: NodeExecution[];
  variables: Record<string, any>;
  logs: ExecutionLog[];
  metrics: ExecutionMetrics;
}
```

### NodeExecution 节点执行模型

```typescript
interface NodeExecution {
  nodeId: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped';
  startedAt?: string;
  completedAt?: string;
  duration?: number;
  input?: any;
  output?: any;
  error?: string;
  metrics: {
    executionTime: number;
    memoryUsage: number;
    cpuUsage: number;
  };
}
```

### Alert 告警模型

```typescript
interface Alert {
  id: string;
  type: 'performance' | 'error' | 'security' | 'system';
  severity: 'low' | 'medium' | 'high' | 'critical';
  status: 'active' | 'acknowledged' | 'resolved';
  title: string;
  message: string;
  workflowId?: string;
  executionId?: string;
  triggeredAt: string;
  acknowledgedAt?: string;
  resolvedAt?: string;
  metadata: Record<string, any>;
}
```

## WebSocket 实时通信

### 连接端点

```
wss://api.hetumind.com/ws/v1/realtime
```

### 认证

```javascript
const ws = new WebSocket('wss://api.hetumind.com/ws/v1/realtime');
ws.onopen = () => {
  // 发送认证消息
  ws.send(
    JSON.stringify({
      type: 'auth',
      token: 'your-bearer-token',
    })
  );
};
```

### 消息格式

#### 订阅工作流执行状态

```json
{
  "type": "subscribe",
  "channel": "execution",
  "executionId": "exec-123"
}
```

#### 执行状态更新

```json
{
  "type": "execution_update",
  "data": {
    "executionId": "exec-123",
    "status": "completed",
    "nodeId": "node-1",
    "output": { "result": "success" }
  },
  "timestamp": "2024-01-25T14:30:00Z"
}
```

#### 系统通知

```json
{
  "type": "notification",
  "data": {
    "id": "notif-123",
    "type": "info",
    "title": "工作流执行完成",
    "message": "工作流 '数据处理' 已成功完成",
    "workflowId": "workflow-123",
    "executionId": "exec-123"
  },
  "timestamp": "2024-01-25T14:30:00Z"
}
```

## 限制和配额

### API 限制

| 资源       | 限制                 |
| ---------- | -------------------- |
| 请求频率   | 1000 请求/小时       |
| 并发连接   | 10 个 WebSocket 连接 |
| 工作流数量 | 1000 个/用户         |
| 执行历史   | 10000 条记录         |
| 文件上传   | 10 MB/文件           |

### 数据限制

| 项目       | 限制          |
| ---------- | ------------- |
| 工作流名称 | 100 字符      |
| 节点数量   | 500 个/工作流 |
| 变量大小   | 1 MB          |
| 日志条目   | 10000 条/执行 |
| 执行超时   | 24 小时       |

## 版本控制

### API 版本

- **当前版本**: v1
- **版本策略**: 语义化版本控制
- **向后兼容**: 保证 v1.x 版本的向后兼容性

### 版本更新通知

API 版本更新时会提前 30 天通过以下方式通知用户：

- 系统内通知
- 邮件通知
- API 响应头 `X-API-Deprecation-Warning`
- 文档更新通知

## SDK 和工具

### JavaScript/TypeScript SDK

```typescript
import { HetumindClient } from '@hetumind/web-sdk';

const client = new HetumindClient({
  baseURL: 'https://api.hetumind.com',
  token: 'your-token',
});

// 创建工作流
const workflow = await client.workflows.create({
  name: '新工作流',
  nodes: [],
  edges: [],
});

// 执行工作流
const execution = await client.workflows.execute(workflow.id);

// 监听执行状态
client.on('execution.update', data => {
  console.log('执行状态更新:', data);
});
```

### CLI 工具

```bash
# 安装 CLI
npm install -g @hetumind/cli

# 认证
hetumind auth login

# 管理工作流
hetumind workflow list
hetumind workflow create
hetumind workflow execute workflow-123

# 监控执行
hetumind execution watch exec-123
hetumind execution logs exec-123
```

---

如有任何问题或建议，请联系我们的技术支持团队或查看 [GitHub Issues](https://github.com/hetu-data/hetumind/issues)。
