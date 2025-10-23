# Hetumind Web - AI Agent 开发和工作流编排平台

<div align="center">

![Hetumind Logo](../src/assets/logo.png)

**Hetumind Web** 是一个现代化的 AI Agent 开发和工作流编排平台，提供直观的可视化界面来设计、构建和管理复杂的 AI 工作流。

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Version](https://img.shields.io/badge/Version-0.1.0-green.svg)](https://github.com/fusion-data/hetumind)
[![React](https://img.shields.io/badge/React-19.1.1-blue.svg)](https://reactjs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.9.2-blue.svg)](https://www.typescriptlang.org/)
[![Ant Design](https://img.shields.io/badge/Ant%20Design-5.27.4-blue.svg)](https://ant.design/)

</div>

## 📋 目录

- [✨ 核心特性](#-核心特性)
- [🚀 快速开始](#-快速开始)
- [🏗️ 系统架构](#️-系统架构)
- [📚 用户指南](#-用户指南)
- [🛠️ 开发指南](#️-开发指南)
- [🧪 测试](#-测试)
- [📊 性能优化](#-性能优化)
- [🚀 部署](#-部署)
- [🤝 贡献指南](#-贡献指南)
- [📄 许可证](#-许可证)

## ✨ 核心特性

### 🎨 可视化工作流编辑器

- **拖拽式设计**: 直观的拖放界面，轻松构建复杂工作流
- **实时预览**: 即时查看工作流执行效果
- **智能连接**: 自动节点连接验证和优化建议
- **版本控制**: 工作流版本管理和回滚功能

### 🤖 AI Agent 集成

- **多模型支持**: 支持 GPT、Claude、本地模型等多种 AI 模型
- **智能配置**: 自动优化 AI 模型参数设置
- **上下文管理**: 智能的对话上下文保持和管理
- **成本控制**: 实时监控和优化 API 调用成本

### ⚡ 高性能执行引擎

- **并发处理**: 支持大规模并发工作流执行
- **智能调度**: 基于优先级和资源的智能任务调度
- **错误恢复**: 自动错误检测和恢复机制
- **实时监控**: 全面的执行状态和性能监控

### 🔧 数据处理能力

- **数据映射**: 可视化数据字段映射和转换
- **表达式引擎**: 强大的表达式计算和条件判断
- **连接器**: 丰富的数据源连接器（数据库、API、文件等）
- **数据验证**: 完整的数据类型和格式验证

### 📊 监控和分析

- **实时仪表板**: 工作流执行状态和系统性能实时监控
- **性能分析**: 详细的性能指标和优化建议
- **日志管理**: 完整的执行日志和审计跟踪
- **告警系统**: 智能的异常检测和告警通知

## 🚀 快速开始

### 环境要求

- **Node.js**: ≥ 22.0.0
- **pnpm**: ≥ 8.0.0
- **TypeScript**: ≥ 5.9.2
- **现代浏览器**: Chrome 90+, Firefox 88+, Safari 14+

### 安装和运行

```bash
# 克隆仓库
git clone https://github.com/fusion-data/hetumind.git
cd hetumind/hetumind-web

# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev

# 构建生产版本
pnpm build

# 预览生产版本
pnpm preview
```

### 第一个工作流

1. **创建新工作流**
   - 点击"新建工作流"按钮
   - 选择工作流模板或从空白开始

2. **添加节点**
   - 从左侧面板拖拽节点到画布
   - 配置节点参数和属性

3. **连接节点**
   - 拖拽连接线创建节点间的数据流
   - 设置连接条件和数据映射

4. **测试执行**
   - 点击"测试运行"按钮
   - 观察执行结果和调试信息

5. **发布工作流**
   - 保存并发布工作流
   - 设置触发条件和执行计划

## 🏗️ 系统架构

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Hetumind Web Platform                   │
├─────────────────────────────────────────────────────────────┤
│  Frontend (React + TypeScript + Ant Design)                │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ 工作流编辑器  │ AI Agent   │ 数据处理    │ 监控面板    │ │
│  │             │ 集成        │            │             │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  State Management (Zustand) + Context API                 │
├─────────────────────────────────────────────────────────────┤
│  Workflow Engine (TypeScript)                             │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ 节点执行器   │ 调度器      │ 监控器      │ 缓存系统    │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Backend Integration (REST API + WebSocket)               │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ Hetuflow    │ Hetumind    │ Fusionsql  │ 认证服务    │ │
│  │ (工作流)     │ (AI Agent)  │ (数据库)    │             │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 技术栈

#### 前端技术

- **React 19.1.1**: 用户界面框架
- **TypeScript 5.9.2**: 类型安全的 JavaScript
- **Ant Design 5.27.4**: 企业级 UI 组件库
- **React Flow 12.8.6**: 工作流可视化
- **Zustand 5.0.2**: 轻量级状态管理
- **React Query 5.62.3**: 数据获取和缓存
- **Recharts 3.2.1**: 数据可视化

#### 开发工具

- **Vite 7.1.7**: 构建工具
- **ESLint 9.36.0**: 代码检查
- **Prettier 3.3.3**: 代码格式化
- **Jest 29.7.0**: 测试框架
- **Testing Library**: React 组件测试

### 核心模块

#### 1. 工作流编辑器 (WorkflowEditor)

```typescript
interface WorkflowEditor {
  // 画布组件
  WorkflowCanvas: React.ComponentType;

  // 节点管理
  NodeFactory: NodeFactory;
  NodeRegistry: NodeRegistry;

  // 连接管理
  ConnectionManager: ConnectionManager;

  // 工具栏
  WorkflowToolbar: React.ComponentType;
}
```

#### 2. 节点系统 (Node System)

```typescript
interface NodeSystem {
  // 基础节点类型
  TriggerNode: React.ComponentType<TriggerNodeProps>;
  AIAgentNode: React.ComponentType<AIAgentNodeProps>;
  ConditionNode: React.ComponentType<ConditionNodeProps>;
  ActionNode: React.ComponentType<ActionNodeProps>;
  DataProcessorNode: React.ComponentType<DataProcessorNodeProps>;

  // 节点执行器
  NodeExecutor: BaseNodeExecutor;
  NodeExecutorRegistry: Map<string, NodeExecutor>;
}
```

#### 3. 执行引擎 (Execution Engine)

```typescript
interface ExecutionEngine {
  // 工作流执行
  WorkflowEngine: class;

  // 执行监控
  ExecutionMonitor: React.ComponentType;

  // 调度器
  TaskScheduler: class;

  // 状态管理
  ExecutionContext: class;
}
```

#### 4. 数据处理 (Data Processing)

```typescript
interface DataProcessing {
  // 数据映射
  DataMappingCanvas: React.ComponentType;

  // 表达式引擎
  ExpressionEngine: class;

  // 数据连接器
  DataConnector: React.ComponentType;

  // 数据验证
  DataValidator: class;
}
```

## 📚 用户指南

### 工作流基础

#### 什么是工作流？

工作流是一个自动化的业务流程，由一系列相互连接的节点组成，每个节点执行特定的任务。

#### 节点类型

1. **触发器节点 (Trigger Nodes)**
   - 手动触发：用户手动启动工作流
   - 定时触发：基于时间计划自动执行
   - Webhook触发：通过 HTTP 请求触发

2. **AI Agent 节点 (AI Agent Nodes)**
   - 对话机器人：处理多轮对话
   - 文本生成：基于提示生成文本
   - 图像生成：创建 AI 生成图像
   - 向量嵌入：文本向量化处理

3. **条件节点 (Condition Nodes)**
   - IF 条件：基于表达式的条件判断
   - SWITCH 条件：多分支条件选择
   - 自定义条件：基于脚本的自定义逻辑

4. **动作节点 (Action Nodes)**
   - API 调用：发送 HTTP 请求
   - 数据库操作：数据库增删改查
   - 邮件发送：自动邮件通知
   - 文件操作：文件读写和处理

5. **数据处理节点 (Data Processing Nodes)**
   - 数据映射：字段映射和转换
   - 数据过滤：基于条件的数据筛选
   - 数据聚合：数据统计和汇总
   - 数据转换：复杂的数据处理逻辑

### 创建工作流

#### 步骤 1：创建新工作流

1. 登录 Hetumind Web 平台
2. 点击"新建工作流"按钮
3. 输入工作流名称和描述
4. 选择工作流模板（可选）
5. 点击"创建"按钮

#### 步骤 2：添加和配置节点

1. **添加节点**
   - 从左侧节点面板拖拽所需节点到画布
   - 或双击节点面板中的节点快速添加

2. **配置节点**
   - 选中画布中的节点
   - 在右侧属性面板中配置节点参数
   - 设置节点名称、描述和其他属性

3. **连接节点**
   - 从源节点的输出端口拖拽到目标节点的输入端口
   - 在连接线上设置数据映射和条件

#### 步骤 3：测试和调试

1. **测试运行**
   - 点击工具栏中的"测试运行"按钮
   - 观察节点的执行状态和结果
   - 检查数据流和错误信息

2. **调试信息**
   - 查看节点执行日志
   - 检查输入输出数据
   - 分析性能指标

#### 步骤 4：发布工作流

1. **保存工作流**
   - 点击"保存"按钮保存当前配置
   - 添加版本说明和变更记录

2. **发布工作流**
   - 点击"发布"按钮发布工作流
   - 设置触发条件和执行计划
   - 配置通知和告警设置

### 高级功能

#### 数据映射

- **字段映射**: 在不同节点间映射数据字段
- **数据转换**: 使用表达式转换数据格式
- **条件映射**: 基于条件进行不同的数据映射
- **批量处理**: 处理数组类型的数据

#### 表达式引擎

- **基础表达式**: 数学运算、字符串操作、逻辑判断
- **函数调用**: 内置函数和自定义函数
- **变量引用**: 引用工作流变量和节点输出
- **模板字符串**: 动态生成文本内容

#### 错误处理

- **重试机制**: 自动重试失败的节点
- **错误捕获**: 捕获和处理执行错误
- **分支处理**: 基于错误结果执行不同的逻辑
- **通知告警**: 错误发生时发送通知

## 🛠️ 开发指南

### 开发环境设置

#### 1. 克隆和安装

```bash
git clone https://github.com/fusion-data/hetumind.git
cd hetumind/hetumind-web
pnpm install
```

#### 2. 环境配置

```bash
# 复制环境配置文件
cp .env.example .env

# 编辑配置文件
vim .env
```

#### 3. 启动开发服务器

```bash
pnpm dev
```

### 项目结构

```
hetumind-web/
├── public/                 # 静态资源
├── src/                    # 源代码
│   ├── components/         # 组件
│   │   ├── workflow/       # 工作流相关组件
│   │   │   ├── nodes/      # 节点组件
│   │   │   ├── execution/  # 执行引擎组件
│   │   │   ├── monitoring/ # 监控组件
│   │   │   └── optimization/ # 性能优化组件
│   │   ├── layout/         # 布局组件
│   │   ├── common/         # 通用组件
│   │   └── ui/             # UI 组件
│   ├── pages/              # 页面组件
│   ├── hooks/              # 自定义 Hooks
│   ├── store/              # 状态管理
│   ├── utils/              # 工具函数
│   ├── types/              # TypeScript 类型定义
│   ├── assets/             # 资源文件
│   └── styles/             # 样式文件
├── docs/                   # 文档
├── scripts/                # 构建和部署脚本
├── tests/                  # 测试文件
└──配置文件
```

### 组件开发

#### 创建新组件

1. **组件文件结构**

```
src/components/workflow/NewComponent/
├── index.ts              # 导出文件
├── NewComponent.tsx      # 主组件文件
├── NewComponent.module.css # 样式文件
├── types.ts              # 类型定义
├── hooks.ts              # 自定义 Hooks
└── __tests__/            # 测试文件
    ├── NewComponent.test.tsx
    └── NewComponent.integration.test.tsx
```

2. **组件模板**

```typescript
// NewComponent.tsx
import React, { useState, useEffect } from 'react';
import { Card, Typography } from 'antd';
import styles from './NewComponent.module.css';

interface NewComponentProps {
  title: string;
  onAction?: () => void;
}

export const NewComponent: React.FC<NewComponentProps> = ({
  title,
  onAction,
}) => {
  const [state, setState] = useState(initialState);

  useEffect(() => {
    // 组件挂载时的逻辑
  }, []);

  return (
    <Card className={styles.container}>
      <Typography.Title level={4}>{title}</Typography>
      {/* 组件内容 */}
    </Card>
  );
};

export default NewComponent;
```

3. **类型定义**

```typescript
// types.ts
export interface NewComponentData {
  id: string;
  name: string;
  config: Record<string, any>;
}

export interface NewComponentProps {
  data: NewComponentData;
  onChange?: (data: NewComponentData) => void;
  readonly?: boolean;
}
```

#### 节点开发指南

1. **基础节点结构**

```typescript
// nodes/CustomNode/CustomNode.tsx
import React from 'react';
import { Handle, Position } from '@xyflow/react';
import { BaseNode } from '../BaseNode';

interface CustomNodeProps {
  id: string;
  data: {
    label: string;
    config: Record<string, any>;
  };
  selected?: boolean;
}

export const CustomNode: React.FC<CustomNodeProps> = ({
  id,
  data,
  selected,
}) => {
  return (
    <div className={`custom-node ${selected ? 'selected' : ''}`}>
      <Handle type="target" position={Position.Left} />
      <div className="node-content">
        <h4>{data.label}</h4>
        {/* 节点配置界面 */}
      </div>
      <Handle type="source" position={Position.Right} />
    </div>
  );
};
```

2. **节点执行器**

```typescript
// nodes/CustomNode/CustomNodeExecutor.ts
import { BaseNodeExecutor } from '../BaseNode';

export class CustomNodeExecutor extends BaseNodeExecutor {
  async execute(context: ExecutionContext): Promise<ExecutionResult> {
    const { config } = context.nodeData;

    try {
      // 执行节点逻辑
      const result = await this.processData(config, context.input);

      return {
        nodeId: context.nodeId,
        status: 'completed',
        output: result,
        timestamp: Date.now(),
      };
    } catch (error) {
      return {
        nodeId: context.nodeId,
        status: 'failed',
        error: error.message,
        timestamp: Date.now(),
      };
    }
  }

  validate(config: any): boolean {
    // 验证配置参数
    return config && typeof config === 'object';
  }

  private async processData(config: any, input: any): Promise<any> {
    // 自定义处理逻辑
    return processedData;
  }
}
```

3. **注册节点**

```typescript
// nodes/index.ts
export { CustomNode } from './CustomNode/CustomNode';
export { CustomNodeExecutor } from './CustomNode/CustomNodeExecutor';

// 在节点注册表中注册
const nodeRegistry = new NodeRegistry();
nodeRegistry.register('custom', CustomNode, CustomNodeExecutor);
```

### 状态管理

#### Zustand Store 结构

```typescript
// store/workflowStore.ts
import { create } from 'zustand';

interface WorkflowState {
  // 状态
  workflows: Workflow[];
  currentWorkflow: Workflow | null;
  selectedNodes: string[];

  // 动作
  setWorkflows: (workflows: Workflow[]) => void;
  setCurrentWorkflow: (workflow: Workflow) => void;
  addNode: (node: WorkflowNode) => void;
  updateNode: (id: string, updates: Partial<WorkflowNode>) => void;
  deleteNode: (id: string) => void;

  // 计算属性
  getNodeById: (id: string) => WorkflowNode | undefined;
  getConnectedNodes: (id: string) => WorkflowNode[];
}

export const useWorkflowStore = create<WorkflowState>((set, get) => ({
  // 初始状态
  workflows: [],
  currentWorkflow: null,
  selectedNodes: [],

  // 实现动作
  setWorkflows: workflows => set({ workflows }),
  setCurrentWorkflow: workflow => set({ currentWorkflow: workflow }),

  addNode: node =>
    set(state => ({
      currentWorkflow: state.currentWorkflow
        ? {
            ...state.currentWorkflow,
            nodes: [...state.currentWorkflow.nodes, node],
          }
        : null,
    })),

  // 实现其他动作...
}));
```

### API 集成

#### 数据获取示例

```typescript
// hooks/useWorkflows.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { workflowAPI } from '../services/api';

export const useWorkflows = () => {
  return useQuery({
    queryKey: ['workflows'],
    queryFn: workflowAPI.getWorkflows,
    staleTime: 5 * 60 * 1000, // 5 分钟
  });
};

export const useCreateWorkflow = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: workflowAPI.createWorkflow,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workflows'] });
    },
  });
};
```

### 样式指南

#### CSS Modules

```css
/* CustomNode.module.css */
.container {
  padding: 16px;
  border: 2px solid #d9d9d9;
  border-radius: 8px;
  background: #ffffff;
  transition: all 0.3s ease;
}

.container.selected {
  border-color: #1890ff;
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}

.nodeContent {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.title {
  font-weight: 600;
  color: #262626;
}
```

#### 主题定制

```typescript
// styles/theme.ts
import { theme } from 'antd';

export const customTheme = {
  ...theme,
  token: {
    ...theme.token,
    colorPrimary: '#1890ff',
    borderRadius: 8,
    fontSize: 14,
  },
  components: {
    Button: {
      borderRadius: 6,
    },
    Card: {
      borderRadius: 8,
    },
  },
};
```

## 🧪 测试

### 测试策略

#### 1. 单元测试

- 组件渲染测试
- 用户交互测试
- 工具函数测试
- 状态管理测试

#### 2. 集成测试

- 组件间交互测试
- API 集成测试
- 工作流执行测试
- 数据流测试

#### 3. 端到端测试

- 完整用户流程测试
- 跨浏览器兼容性测试
- 性能测试
- 可访问性测试

### 运行测试

```bash
# 运行所有测试
npm run test

# 运行单元测试
npm run test:unit

# 运行集成测试
npm run test:integration

# 生成覆盖率报告
npm run test:coverage

# 监视模式
npm run test:watch
```

### 测试示例

```typescript
// __tests__/WorkflowCanvas.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { WorkflowCanvas } from '../WorkflowCanvas';

describe('WorkflowCanvas', () => {
  it('renders workflow canvas', () => {
    render(<WorkflowCanvas />);
    expect(screen.getByTestId('workflow-canvas')).toBeInTheDocument();
  });

  it('adds node when button clicked', () => {
    const onNodesChange = jest.fn();
    render(<WorkflowCanvas onNodesChange={onNodesChange} />);

    fireEvent.click(screen.getByTestId('add-node'));
    expect(onNodesChange).toHaveBeenCalled();
  });
});
```

## 📊 性能优化

### 性能监控

#### 1. 渲染性能

- 组件渲染时间监控
- 重渲染优化
- 虚拟化长列表

#### 2. 内存管理

- 内存泄漏检测
- 组件卸载清理
- 大对象处理优化

#### 3. 网络优化

- API 请求缓存
- 数据分页加载
- 并发请求控制

### 优化策略

#### 1. React 优化

```typescript
// 使用 React.memo
const OptimizedComponent = React.memo(({ data }) => {
  return <div>{data.title}</div>;
});

// 使用 useMemo 和 useCallback
const Component = ({ items, onItemClick }) => {
  const expensiveValue = useMemo(() => {
    return items.reduce((sum, item) => sum + item.value, 0);
  }, [items]);

  const handleClick = useCallback((item) => {
    onItemClick(item);
  }, [onItemClick]);

  return <div>{/* 组件内容 */}</div>;
};
```

#### 2. 工作流引擎优化

```typescript
// 批量处理节点
class WorkflowEngine {
  private async processBatch(nodes: WorkflowNode[]) {
    const BATCH_SIZE = 10;
    for (let i = 0; i < nodes.length; i += BATCH_SIZE) {
      const batch = nodes.slice(i, i + BATCH_SIZE);
      await Promise.all(batch.map(node => this.processNode(node)));
    }
  }
}
```

#### 3. 缓存策略

```typescript
// 使用 React Query 缓存
const useWorkflows = () => {
  return useQuery({
    queryKey: ['workflows'],
    queryFn: fetchWorkflows,
    staleTime: 5 * 60 * 1000, // 5 分钟缓存
    cacheTime: 10 * 60 * 1000, // 10 分钟存储
  });
};
```

## 🚀 部署

### 构建配置

#### 1. 生产构建

```bash
# 构建生产版本
npm run build

# 分析构建包大小
npm run build:analyze
```

#### 2. 环境配置

```typescript
// config/environments.ts
export const environments = {
  development: {
    API_BASE_URL: 'http://localhost:3001',
    WEBSOCKET_URL: 'ws://localhost:3001',
  },
  production: {
    API_BASE_URL: 'https://api.hetumind.com',
    WEBSOCKET_URL: 'wss://api.hetumind.com',
  },
};
```

### Docker 部署

#### Dockerfile

```dockerfile
# 多阶段构建
FROM node:22-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

# 生产镜像
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

#### docker-compose.yml

```yaml
version: '3.8'

services:
  hetumind-web:
    build: .
    ports:
      - '80:80'
    environment:
      - NODE_ENV=production
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    restart: unless-stopped
```

### CI/CD 配置

#### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-node@v2
        with:
          node-version: '22'
      - run: npm ci
      - run: npm run test:ci
      - run: npm run build

  deploy:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Deploy to production
        run: |
          # 部署脚本
```

## 🤝 贡献指南

### 开发流程

1. **Fork 项目**

   ```bash
   git clone https://github.com/your-username/hetumind.git
   ```

2. **创建功能分支**

   ```bash
   git checkout -b feature/amazing-feature
   ```

3. **提交更改**

   ```bash
   git commit -m 'Add amazing feature'
   ```

4. **推送分支**

   ```bash
   git push origin feature/amazing-feature
   ```

5. **创建 Pull Request**

### 代码规范

#### 1. TypeScript 规范

```typescript
// ✅ 好的实践
interface UserData {
  id: string;
  name: string;
  email: string;
}

const getUser = async (id: string): Promise<UserData> => {
  const response = await fetch(`/api/users/${id}`);
  return response.json();
};

// ❌ 避免的实践
const getUser = id => {
  return fetch(`/api/users/${id}`);
};
```

#### 2. React 组件规范

```typescript
// ✅ 函数组件 + TypeScript
interface ComponentProps {
  title: string;
  onSubmit?: (data: FormData) => void;
}

export const Component: React.FC<ComponentProps> = ({
  title,
  onSubmit,
}) => {
  const [data, setData] = useState<FormData>(initialData);

  const handleSubmit = useCallback(() => {
    onSubmit?.(data);
  }, [data, onSubmit]);

  return (
    <div>
      <h2>{title}</h2>
      {/* 组件内容 */}
    </div>
  );
};
```

#### 3. 样式规范

```typescript
// ✅ CSS Modules + TypeScript
import styles from './Component.module.css';

const Component = () => {
  return <div className={styles.container}>Content</div>;
};
```

### 提交规范

使用 [Conventional Commits](https://conventionalcommits.org/) 规范：

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

类型说明：

- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式化
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建工具或辅助工具的变动

示例：

```
feat(workflow): add AI agent node integration

- Implement AI agent configuration panel
- Add support for multiple AI models
- Include cost monitoring features

Closes #123
```

## 📄 许可证

本项目采用 [Apache License 2.0](LICENSE) 许可证。

## 📞 联系我们

- **项目主页**: https://github.com/fusion-data/hetumind
- **文档网站**: https://docs.hetumind.com
- **问题反馈**: https://github.com/fusion-data/hetumind/issues
- **讨论区**: https://github.com/fusion-data/hetumind/discussions

---

⭐ 如果这个项目对您有帮助，请给我们一个 Star！
