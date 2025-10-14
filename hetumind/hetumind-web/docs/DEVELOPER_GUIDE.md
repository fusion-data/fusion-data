# Hetumind Web 开发者指南

本文档面向开发人员，提供 Hetumind Web 平台的技术架构、开发环境、代码规范和贡献指南。

## 📋 目录

- [技术架构](#技术架构)
- [开发环境](#开发环境)
- [项目结构](#项目结构)
- [核心概念](#核心概念)
- [组件开发](#组件开发)
- [状态管理](#状态管理)
- [API 集成](#api-集成)
- [测试指南](#测试指南)
- [性能优化](#性能优化)
- [部署指南](#部署指南)
- [贡献流程](#贡献流程)

## 技术架构

### 技术栈

#### 前端技术
- **React 19.1.1**: 现代化的用户界面框架
- **TypeScript 5.9.2**: 类型安全的 JavaScript 超集
- **Ant Design 5.27.4**: 企业级 UI 组件库
- **React Flow 12.8.6**: 工作流可视化组件
- **Zustand 5.0.2**: 轻量级状态管理
- **React Query 5.62.3**: 数据获取和缓存
- **Recharts 3.2.1**: 数据可视化图表库
- **Vite 7.1.7**: 现代化的构建工具

#### 开发工具
- **ESLint 9.36.0**: 代码质量检查
- **Prettier 3.3.3**: 代码格式化
- **Jest 29.7.0**: 单元测试框架
- **Testing Library**: React 组件测试
- **Husky**: Git hooks 管理
- **lint-staged**: 暂存文件检查

### 架构模式

#### 分层架构
```
┌─────────────────────────────────────────────────────────────┐
│                    表现层 (Presentation Layer)                │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ 页面组件    │ 业务组件    │ 通用组件    │ 布局组件    │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    业务层 (Business Layer)                   │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ 工作流逻辑   │ AI 集成     │ 数据处理    │ 监控逻辑    │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    数据层 (Data Layer)                       │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ 状态管理     │ API 客户端   │ 缓存管理     │ 本地存储    │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    基础设施层 (Infrastructure Layer)          │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ HTTP 客户端  │ WebSocket   │ 工具函数     │ 类型定义    │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

#### 设计模式
- **组合模式**: 组件的灵活组合和复用
- **观察者模式**: 状态变化的响应式更新
- **策略模式**: 不同算法和策略的可插拔设计
- **工厂模式**: 节点和组件的统一创建
- **适配器模式**: 不同接口的适配和转换

### 核心模块

#### 1. 工作流引擎 (Workflow Engine)
```typescript
interface WorkflowEngine {
  // 工作流定义和管理
  WorkflowDefinition: class;
  WorkflowNode: interface;
  WorkflowEdge: interface;

  // 执行引擎
  ExecutionEngine: class;
  NodeExecutor: interface;
  ExecutionContext: class;

  // 状态管理
  WorkflowStore: interface;
  ExecutionStore: interface;
}
```

#### 2. 节点系统 (Node System)
```typescript
interface NodeSystem {
  // 节点组件
  BaseNode: React.Component;
  TriggerNode: React.Component;
  AIAgentNode: React.Component;
  ConditionNode: React.Component;
  ActionNode: React.Component;

  // 节点工厂
  NodeFactory: class;
  NodeRegistry: class;
  NodeRenderer: React.Component;

  // 节点执行器
  BaseNodeExecutor: class;
  NodeExecutorMap: Map<string, BaseNodeExecutor>;
}
```

#### 3. 数据处理 (Data Processing)
```typescript
interface DataProcessing {
  // 数据映射
  DataMappingCanvas: React.Component;
  MappingRule: interface;
  DataTransformer: class;

  // 表达式引擎
  ExpressionEngine: class;
  ExpressionParser: class;
  ExpressionEvaluator: class;

  // 数据连接器
  DataConnector: React.Component;
  ConnectionManager: class;
}
```

## 开发环境

### 环境要求

#### 必需软件
- **Node.js**: >= 22.0.0
- **pnpm**: >= 8.0.0
- **Git**: >= 2.30.0
- **VS Code**: >= 1.85.0 (推荐)

#### 可选软件
- **Chrome**: >= 90.0 (开发调试)
- **Docker**: >= 20.0 (容器化部署)
- **Postman**: >= 10.0 (API 测试)

### 环境配置

#### 1. 克隆项目
```bash
git clone https://github.com/fusion-data/hetumind.git
cd hetumind/hetumind-web
```

#### 2. 安装依赖
```bash
# 安装 pnpm (如果没有安装)
npm install -g pnpm

# 安装项目依赖
pnpm install
```

#### 3. 环境配置
```bash
# 复制环境配置文件
cp .env.example .env

# 编辑环境变量
vim .env
```

#### 4. 启动开发服务器
```bash
# 启动开发服务器
pnpm dev

# 启动开发服务器并自动打开浏览器
pnpm dev --open

# 指定端口启动
pnpm dev --port 3000
```

### VS Code 配置

#### 推荐扩展
```json
{
  "recommendations": [
    "ms-vscode.vscode-typescript-next",
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode",
    "ms-vscode.vscode-eslint",
    "formulahendry.auto-rename-tag",
    "christian-kohler.path-intellisense",
    "ms-vscode.vscode-jest"
  ]
}
```

#### 工作区设置
```json
{
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": "explicit"
  },
  "typescript.preferences.importModuleSpecifier": "relative",
  "emmet.includeLanguages": {
    "typescript": "html",
    "typescriptreact": "html"
  }
}
```

### 开发脚本

#### package.json 脚本
```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint src --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
    "lint:fix": "eslint src --ext ts,tsx --fix",
    "type-check": "tsc --noEmit",
    "test": "node scripts/test-runner.js",
    "test:unit": "node scripts/test-runner.js unit",
    "test:integration": "node scripts/test-runner.js integration",
    "test:coverage": "jest --coverage",
    "test:watch": "jest --watch",
    "storybook": "storybook dev -p 6006",
    "build-storybook": "storybook build"
  }
}
```

## 项目结构

### 目录结构
```
hetumind-web/
├── public/                 # 静态资源
│   ├── favicon.ico
│   ├── logo.png
│   └── manifest.json
├── src/                    # 源代码
│   ├── components/         # 组件
│   │   ├── workflow/       # 工作流组件
│   │   │   ├── nodes/      # 节点组件
│   │   │   │   ├── BaseNode/
│   │   │   │   ├── TriggerNode/
│   │   │   │   ├── AIAgentNode/
│   │   │   │   └── index.ts
│   │   │   ├── execution/  # 执行组件
│   │   │   ├── monitoring/ # 监控组件
│   │   │   ├── optimization/ # 性能优化
│   │   │   ├── panel/      # 面板组件
│   │   │   ├── toolbar/    # 工具栏组件
│   │   │   └── WorkflowEditor.tsx
│   │   ├── layout/         # 布局组件
│   │   ├── common/         # 通用组件
│   │   └── ui/             # UI 组件
│   ├── pages/              # 页面组件
│   │   ├── Dashboard/
│   │   ├── WorkflowEditor/
│   │   ├── Monitoring/
│   │   └── Settings/
│   ├── hooks/              # 自定义 Hooks
│   │   ├── useWorkflow.ts
│   │   ├── useExecution.ts
│   │   └── usePerformance.ts
│   ├── store/              # 状态管理
│   │   ├── workflowStore.ts
│   │   ├── executionStore.ts
│   │   └── index.ts
│   ├── services/           # 服务层
│   │   ├── api/
│   │   ├── websocket/
│   │   └── storage/
│   ├── utils/              # 工具函数
│   │   ├── helpers.ts
│   │   ├── validators.ts
│   │   └── constants.ts
│   ├── types/              # 类型定义
│   │   ├── workflow.ts
│   │   ├── execution.ts
│   │   └── api.ts
│   ├── assets/             # 静态资源
│   │   ├── images/
│   │   ├── icons/
│   │   └── styles/
│   ├── styles/             # 样式文件
│   │   ├── globals.css
│   │   ├── variables.css
│   │   └── components.css
│   ├── App.tsx             # 应用根组件
│   ├── main.tsx            # 应用入口
│   └── vite-env.d.ts       # Vite 环境声明
├── docs/                   # 文档
├── scripts/                # 构建脚本
├── tests/                  # 测试文件
├── .env.example           # 环境变量示例
├── .gitignore             # Git 忽略文件
├── .eslintrc.js           # ESLint 配置
├── .prettierrc            # Prettier 配置
├── jest.config.js         # Jest 配置
├── tsconfig.json          # TypeScript 配置
├── vite.config.ts         # Vite 配置
└── package.json           # 项目配置
```

### 文件命名规范

#### 组件文件
```
组件名：PascalCase
文件名：PascalCase.tsx
样式文件：PascalCase.module.css
类型文件：types.ts
测试文件：ComponentName.test.tsx
```

#### 工具文件
```
函数名：camelCase
文件名：camelCase.ts
常量文件：constants.ts
类型文件：types.ts
```

#### 目录命名
```
目录名：kebab-case
组件目录：PascalCase/
工具目录：camelCase/
类型目录：types/
```

## 核心概念

### React 组件

#### 函数组件模式
```typescript
interface ComponentProps {
  title: string;
  onSubmit?: (data: FormData) => void;
  readonly?: boolean;
}

export const Component: React.FC<ComponentProps> = ({
  title,
  onSubmit,
  readonly = false,
}) => {
  const [data, setData] = useState<FormData>(initialData);

  const handleSubmit = useCallback(() => {
    onSubmit?.(data);
  }, [data, onSubmit]);

  return (
    <div className={styles.container}>
      <h2>{title}</h2>
      {/* 组件内容 */}
    </div>
  );
};

export default Component;
```

#### 组件组合模式
```typescript
// 高阶组件
const withWorkflowProvider = <P extends object>(
  Component: React.ComponentType<P>
) => {
  return (props: P) => (
    <WorkflowProvider>
      <Component {...props} />
    </WorkflowProvider>
  );
};

// 组合组件
const WorkflowEditor = () => {
  return (
    <WorkflowProvider>
      <WorkflowLayout>
        <WorkflowToolbar />
        <WorkflowCanvas />
        <PropertyPanel />
      </WorkflowLayout>
    </WorkflowProvider>
  );
};
```

### 自定义 Hooks

#### 数据获取 Hook
```typescript
interface UseWorkflowsOptions {
  enabled?: boolean;
  retry?: boolean;
  staleTime?: number;
}

export const useWorkflows = (options: UseWorkflowsOptions = {}) => {
  const {
    enabled = true,
    retry = true,
    staleTime = 5 * 60 * 1000, // 5 分钟
  } = options;

  return useQuery({
    queryKey: ['workflows'],
    queryFn: workflowAPI.getWorkflows,
    enabled,
    retry,
    staleTime,
  });
};
```

#### 状态管理 Hook
```typescript
interface UseWorkflowState {
  workflows: Workflow[];
  currentWorkflow: Workflow | null;
  isLoading: boolean;
  error: string | null;
}

export const useWorkflowState = (): UseWorkflowState => {
  const store = useWorkflowStore();

  const workflows = store.workflows;
  const currentWorkflow = store.currentWorkflow;
  const isLoading = store.isLoading;
  const error = store.error;

  return {
    workflows,
    currentWorkflow,
    isLoading,
    error,
  };
};
```

### 类型系统

#### 基础类型定义
```typescript
// 工作流相关类型
export interface Workflow {
  id: string;
  name: string;
  description?: string;
  status: WorkflowStatus;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  variables: Record<string, any>;
  settings: WorkflowSettings;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
}

export type WorkflowStatus = 'draft' | 'active' | 'inactive' | 'archived';

// 节点相关类型
export interface WorkflowNode {
  id: string;
  type: string;
  position: Position;
  data: NodeData;
  inputs: string[];
  outputs: string[];
}

export interface Position {
  x: number;
  y: number;
}

export interface NodeData {
  label: string;
  description?: string;
  config: Record<string, any>;
  [key: string]: any;
}
```

#### 高级类型
```typescript
// 条件类型
type ConditionalType<T, U, V> = T extends U ? V : never;

// 工具类型
type PartialByKeys<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

// 深度只读类型
type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P];
};

// 联合类型转交叉类型
type UnionToIntersection<U> = (U extends any ? (k: U) => void : never) extends (
  k: infer I
) ? void extends I
  ? never
  : I;
```

## 组件开发

### 组件设计原则

#### 1. 单一职责
- 每个组件只负责一个功能
- 保持组件的简洁和可读性
- 避免组件过于复杂

#### 2. 可复用性
- 设计通用和可配置的组件
- 使用 props 进行参数化
- 避免硬编码业务逻辑

#### 3. 可测试性
- 组件逻辑与展示分离
- 使用依赖注入
- 编写单元测试

#### 4. 性能优化
- 使用 React.memo 优化渲染
- 合理使用 useMemo 和 useCallback
- 避免不必要的重渲染

### 组件模板

#### 基础组件模板
```typescript
// ComponentName/ComponentName.tsx
import React, { useState, useCallback, useEffect } from 'react';
import { Card, Typography } from 'antd';
import styles from './ComponentName.module.css';

interface ComponentNameProps {
  title: string;
  data: ComponentData;
  onChange?: (data: ComponentData) => void;
  readonly?: boolean;
}

interface ComponentData {
  id: string;
  name: string;
  value: any;
}

export const ComponentName: React.FC<ComponentNameProps> = ({
  title,
  data,
  onChange,
  readonly = false,
}) => {
  const [internalData, setInternalData] = useState<ComponentData>(data);
  const [loading, setLoading] = useState(false);

  // 更新内部状态
  const handleChange = useCallback(
    (newData: Partial<ComponentData>) => {
      const updatedData = { ...internalData, ...newData };
      setInternalData(updatedData);
      onChange?.(updatedData);
    },
    [internalData, onChange]
  );

  // 初始化和清理
  useEffect(() => {
    setInternalData(data);
  }, [data]);

  // 渲染组件
  return (
    <Card className={styles.container}>
      <Typography.Title level={4}>{title}</Typography>
      <div className={styles.content}>
        {/* 组件内容 */}
      </div>
    </Card>
  );
};

export default ComponentName;
```

#### 组件样式
```css
/* ComponentName.module.css */
.container {
  padding: 16px;
  border-radius: 8px;
  background: #ffffff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.container:hover {
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .container {
    padding: 12px;
  }
}
```

#### 组件测试
```typescript
// ComponentName.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { ComponentName } from './ComponentName';

describe('ComponentName', () => {
  const defaultProps = {
    title: 'Test Component',
    data: { id: '1', name: 'Test', value: 'test' },
  };

  it('renders correctly', () => {
    render(<ComponentName {...defaultProps} />);
    expect(screen.getByText('Test Component')).toBeInTheDocument();
  });

  it('handles data changes', () => {
    const handleChange = jest.fn();
    render(
      <ComponentName {...defaultProps} onChange={handleChange} />
    );

    // 模拟数据变更
    const input = screen.getByLabelText('Name');
    fireEvent.change(input, { target: { value: 'Updated Name' } });

    expect(handleChange).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'Updated Name' })
    );
  });
});
```

### 节点开发

#### 节点基类
```typescript
// BaseNode/BaseNode.tsx
import React from 'react';
import { Handle, Position } from '@xyflow/react';
import { BaseNodeProps } from '../types';

export const BaseNode: React.FC<BaseNodeProps> = ({
  id,
  data,
  selected,
  children,
}) => {
  return (
    <div className={`base-node ${selected ? 'selected' : ''}`}>
      <Handle type="target" position={Position.Left} />
      <div className="node-content">
        <div className="node-header">
          <span className="node-icon">{data.icon}</span>
          <span className="node-title">{data.label}</span>
        </div>
        <div className="node-body">{children}</div>
      </div>
      <Handle type="source" position={Position.Right} />
    </div>
  );
};
```

#### 具体节点实现
```typescript
// TriggerNode/TriggerNode.tsx
import React from 'react';
import { BaseNode } from '../BaseNode';
import { TriggerNodeProps } from './types';

export const TriggerNode: React.FC<TriggerNodeProps> = ({
  id,
  data,
  selected,
}) => {
  const getIcon = () => {
    switch (data.triggerType) {
      case 'manual':
        return '👆';
      case 'scheduled':
        return '⏰';
      case 'webhook':
        return '🔌';
      default:
        return '⚡';
    }
  };

  return (
    <BaseNode id={id} data={{ ...data, icon: getIcon() }} selected={selected}>
      <div className="trigger-details">
        <span>类型: {data.triggerType}</span>
        {data.triggerType === 'scheduled' && (
          <span>时间: {data.config.cronExpression}</span>
        )}
      </div>
    </BaseNode>
  );
};
```

#### 节点注册
```typescript
// NodeRegistry/index.ts
import { NodeRegistry } from './NodeRegistry';
import { TriggerNode } from '../nodes/TriggerNode';
import { AIAgentNode } from '../nodes/AIAgentNode';

// 创建节点注册表
const nodeRegistry = new NodeRegistry();

// 注册节点类型
nodeRegistry.register('trigger', TriggerNode, {
  category: 'trigger',
  displayName: '触发器',
  description: '工作流触发节点',
  icon: 'thunderbolt',
  configSchema: {
    type: 'object',
    properties: {
      triggerType: {
        type: 'string',
        enum: ['manual', 'scheduled', 'webhook'],
        default: 'manual',
      },
    },
  },
});

nodeRegistry.register('aiAgent', AIAgentNode, {
  category: 'ai',
  displayName: 'AI Agent',
  description: 'AI 智能处理节点',
  icon: 'robot',
});

export default nodeRegistry;
```

## 状态管理

### Zustand Store 设计

#### 工作流状态管理
```typescript
// store/workflowStore.ts
import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

interface WorkflowState {
  // 状态
  workflows: Workflow[];
  currentWorkflow: Workflow | null;
  selectedNodes: string[];
  isLoading: boolean;
  error: string | null;

  // 动作
  setWorkflows: (workflows: Workflow[]) => void;
  setCurrentWorkflow: (workflow: Workflow) => void;
  addNode: (node: WorkflowNode) => void;
  updateNode: (id: string, updates: Partial<WorkflowNode>) => void;
  deleteNode: (id: string) => void;
  selectNodes: (nodeIds: string[]) => void;
  clearSelection: () => void;

  // 副作用
  reset: () => void;
}

export const useWorkflowStore = create<WorkflowState>()(
  devtools(
    persist(
      (set, get) => ({
        // 初始状态
        workflows: [],
        currentWorkflow: null,
        selectedNodes: [],
        isLoading: false,
        error: null,

        // 动作实现
        setWorkflows: (workflows) => set({ workflows }),

        setCurrentWorkflow: (workflow) => set({ currentWorkflow: workflow }),

        addNode: (node) => set((state) => ({
          currentWorkflow: state.currentWorkflow
            ? {
                ...state.currentWorkflow,
                nodes: [...state.currentWorkflow.nodes, node],
              }
            : null,
        })),

        updateNode: (id, updates) => set((state) => ({
          currentWorkflow: state.currentWorkflow
            ? {
                ...state.currentWorkflow,
                nodes: state.currentWorkflow.nodes.map((node) =>
                  node.id === id ? { ...node, ...updates } : node
                ),
              }
            : null,
        })),

        deleteNode: (id) => set((state) => ({
          currentWorkflow: state.currentWorkflow
            ? {
                ...state.currentWorkflow,
                nodes: state.currentWorkflow.nodes.filter((node) => node.id !== id),
                edges: state.currentWorkflow.edges.filter(
                  (edge) => edge.source !== id && edge.target !== id
                ),
              }
            : null,
        })),

        selectNodes: (nodeIds) => set({ selectedNodes: nodeIds }),
        clearSelection: () => set({ selectedNodes: [] }),

        reset: () => ({
          workflows: [],
          currentWorkflow: null,
          selectedNodes: [],
          isLoading: false,
          error: null,
        }),
      }),
      {
        name: 'workflow-store',
        partialize: (state) => ({
          workflows: state.workflows,
          currentWorkflow: state.currentWorkflow,
        }),
      }
    )
  )
);
```

#### 执行状态管理
```typescript
// store/executionStore.ts
import { create } from 'zustand';
import { Execution } from '../types';

interface ExecutionState {
  executions: Execution[];
  currentExecution: Execution | null;
  isMonitoring: boolean;

  // WebSocket 连接
  wsConnection: WebSocket | null;

  // 动作
  setExecutions: (executions: Execution[]) => void;
  setCurrentExecution: (execution: Execution) => void;
  updateExecution: (id: string, updates: Partial<Execution>) => void;
  startMonitoring: () => void;
  stopMonitoring: () => void;
}

export const useExecutionStore = create<ExecutionState>((set, get) => ({
  executions: [],
  currentExecution: null,
  isMonitoring: false,
  wsConnection: null,

  setExecutions: (executions) => set({ executions }),

  setCurrentExecution: (execution) => set({ currentExecution: execution }),

  updateExecution: (id, updates) => set((state) => ({
    executions: state.executions.map((execution) =>
      execution.executionId === id
        ? { ...execution, ...updates }
        : execution
    ),
    currentExecution:
      state.currentExecution?.executionId === id
        ? { ...state.currentExecution, ...updates }
        : state.currentExecution,
  })),

  startMonitoring: () => {
    const ws = new WebSocket('ws://localhost:3001/ws');

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      get().updateExecution(data.executionId, data);
    };

    set({ wsConnection: ws, isMonitoring: true });
  },

  stopMonitoring: () => {
    const { wsConnection } = get();
    if (wsConnection) {
      wsConnection.close();
    }
    set({ wsConnection: null, isMonitoring: false });
  },
}));
```

### Context API 使用

#### React Context
```typescript
// context/WorkflowContext.tsx
import React, { createContext, useContext, ReactNode } from 'react';
import { useWorkflowStore } from '../store/workflowStore';

interface WorkflowContextValue {
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
}

const WorkflowContext = createContext<WorkflowContextValue | null>(null);

interface WorkflowProviderProps {
  children: ReactNode;
}

export const WorkflowProvider: React.FC<WorkflowProviderProps> = ({
  children,
}) => {
  const store = useWorkflowStore();

  const contextValue: WorkflowContextValue = {
    workflows: store.workflows,
    currentWorkflow: store.currentWorkflow,
    selectedNodes: store.selectedNodes,
    setWorkflows: store.setWorkflows,
    setCurrentWorkflow: store.setCurrentWorkflow,
    addNode: store.addNode,
    updateNode: store.updateNode,
    deleteNode: store.deleteNode,
  };

  return (
    <WorkflowContext.Provider value={contextValue}>
      {children}
    </WorkflowContext.Provider>
  );
};

export const useWorkflow = (): WorkflowContextValue => {
  const context = useContext(WorkflowContext);
  if (!context) {
    throw new Error('useWorkflow must be used within WorkflowProvider');
  }
  return context;
};
```

## API 集成

### HTTP 客户端

#### Axios 配置
```typescript
// services/api/client.ts
import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';

class ApiClient {
  private client: AxiosInstance;

  constructor(baseURL: string) {
    this.client = axios.create({
      baseURL,
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    this.setupInterceptors();
  }

  private setupInterceptors() {
    // 请求拦截器
    this.client.interceptors.request.use(
      (config) => {
        const token = localStorage.getItem('token');
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
      },
      (error) => Promise.reject(error)
    );

    // 响应拦截器
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          // 处理认证失败
          localStorage.removeItem('token');
          window.location.href = '/login';
        }
        return Promise.reject(error);
      }
    );
  }

  // GET 请求
  async get<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.get(url, config);
    return response.data;
  }

  // POST 请求
  async post<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.post(url, data, config);
    return response.data;
  }

  // PUT 请求
  async put<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.put(url, data, config);
    return response.data;
  }

  // DELETE 请求
  async delete<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.delete(url, config);
    return response.data;
  }
}

export const apiClient = new ApiClient(process.env.REACT_APP_API_URL);
```

#### API 服务
```typescript
// services/api/workflows.ts
import { apiClient } from './client';
import { Workflow, CreateWorkflowRequest, UpdateWorkflowRequest } from '../../types';

export const workflowAPI = {
  // 获取工作流列表
  getWorkflows: async (params?: {
    page?: number;
    limit?: number;
    search?: string;
    status?: string;
  }) => {
    const response = await apiClient.get<{
      workflows: Workflow[];
      pagination: {
        page: number;
        limit: number;
        total: number;
        totalPages: number;
      };
    }>('/workflows', { params });
    return response;
  },

  // 获取工作流详情
  getWorkflow: async (id: string) => {
    return apiClient.get<Workflow>(`/workflows/${id}`);
  },

  // 创建工作流
  createWorkflow: async (data: CreateWorkflowRequest) => {
    return apiClient.post<Workflow>('/workflows', data);
  },

  // 更新工作流
  updateWorkflow: async (id: string, data: UpdateWorkflowRequest) => {
    return apiClient.put<Workflow>(`/workflows/${id}`, data);
  },

  // 删除工作流
  deleteWorkflow: async (id: string) => {
    return apiClient.delete(`/workflows/${id}`);
  },

  // 执行工作流
  executeWorkflow: async (id: string, data?: {
    variables?: Record<string, any>;
    options?: {
      timeout?: number;
      enableLogging?: boolean;
    };
  }) => {
    return apiClient.post<{
      executionId: string;
      status: string;
      startedAt: string;
    }>(`/workflows/${id}/execute`, data);
  },
};
```

### React Query 集成

#### 查询 Hooks
```typescript
// hooks/useWorkflows.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { workflowAPI } from '../services/api/workflows';
import { Workflow } from '../types';

export const useWorkflows = (params?: {
  page?: number;
  limit?: number;
  search?: string;
  status?: string;
}) => {
  return useQuery({
    queryKey: ['workflows', params],
    queryFn: () => workflowAPI.getWorkflows(params),
    staleTime: 5 * 60 * 1000, // 5 分钟
    cacheTime: 10 * 60 * 1000, // 10 分钟
  });
};

export const useWorkflow = (id: string) => {
  return useQuery({
    queryKey: ['workflow', id],
    queryFn: () => workflowAPI.getWorkflow(id),
    enabled: !!id,
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

export const useUpdateWorkflow = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: any }) =>
      workflowAPI.updateWorkflow(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['workflows'] });
      queryClient.invalidateQueries({ queryKey: ['workflow', variables.id] });
    },
  });
};

export const useDeleteWorkflow = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: workflowAPI.deleteWorkflow,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workflows'] });
    },
  });
};
```

### WebSocket 集成

#### WebSocket Hook
```typescript
// hooks/useWebSocket.ts
import { useEffect, useRef, useState } from 'react';

interface WebSocketHookOptions {
  url: string;
  onMessage?: (data: any) => void;
  onOpen?: () => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
  reconnectAttempts?: number;
  reconnectInterval?: number;
}

export const useWebSocket = ({
  url,
  onMessage,
  onOpen,
  onClose,
  onError,
  reconnectAttempts = 3,
  reconnectInterval = 1000,
}: WebSocketHookOptions) => {
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectCountRef = useRef(0);

  const connect = () => {
    try {
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        setIsConnected(true);
        setError(null);
        reconnectCountRef.current = 0;
        onOpen?.();
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          onMessage?.(data);
        } catch (err) {
          console.error('WebSocket message parse error:', err);
        }
      };

      ws.onclose = () => {
        setIsConnected(false);
        onClose?.();

        // 自动重连
        if (reconnectCountRef.current < reconnectAttempts) {
          setTimeout(() => {
            reconnectCountRef.current++;
            connect();
          }, reconnectInterval);
        }
      };

      ws.onerror = (event) => {
        setError('WebSocket connection error');
        onError?.(event);
      };
    } catch (err) {
      setError('Failed to create WebSocket connection');
      onError?.(err as Event);
    }
  };

  useEffect(() => {
    connect();

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [url]);

  const sendMessage = (message: any) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    }
  };

  return {
    isConnected,
    error,
    sendMessage,
  };
};
```

## 测试指南

### 测试策略

#### 1. 单元测试
- **组件测试**: 测试组件的渲染和交互
- **Hook 测试**: 测试自定义 Hook 的逻辑
- **工具函数测试**: 测试纯函数的正确性
- **覆盖率要求**: 80% 以上

#### 2. 集成测试
- **组件集成**: 测试组件间的协作
- **API 集成**: 测试 API 调用和数据处理
- **状态管理**: 测试状态管理逻辑
- **工作流集成**: 测试完整的工作流执行

#### 3. 端到端测试
- **用户流程**: 测试完整的用户操作流程
- **跨浏览器**: 测试不同浏览器的兼容性
- **性能测试**: 测试应用的性能表现
- **可访问性**: 测试无障碍访问

### 测试工具

#### Jest 配置
```javascript
// jest.config.js
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'jsdom',
  setupFilesAfterEnv: ['<rootDir>/src/setupTests.ts'],
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },
  collectCoverageFrom: [
    'src/**/*.{ts,tsx}',
    '!src/**/*.d.ts',
    '!src/**/*.stories.tsx',
  ],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80,
    },
  },
};
```

#### Testing Library 配置
```typescript
// src/setupTests.ts
import '@testing-library/jest-dom';
import { configure } from '@testing-library/react';

// 全局配置
configure({ testIdAttribute: 'data-testid' });

// Mock IntersectionObserver
global.IntersectionObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));

// Mock ResizeObserver
global.ResizeObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));
```

### 测试示例

#### 组件测试
```typescript
// components/__tests__/WorkflowCanvas.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { WorkflowCanvas } from '../WorkflowCanvas';

describe('WorkflowCanvas', () => {
  const defaultProps = {
    nodes: [],
    edges: [],
    onNodesChange: jest.fn(),
    onEdgesChange: jest.fn(),
    onConnect: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders workflow canvas', () => {
    render(<WorkflowCanvas {...defaultProps} />);

    expect(screen.getByTestId('workflow-canvas')).toBeInTheDocument();
    expect(screen.getByTestId('react-flow')).toBeInTheDocument();
  });

  it('adds node when add button clicked', async () => {
    const onNodesChange = jest.fn();
    render(
      <WorkflowCanvas {...defaultProps} onNodesChange={onNodesChange} />
    );

    const addButton = screen.getByTestId('add-node');
    await userEvent.click(addButton);

    expect(onNodesChange).toHaveBeenCalledWith(
      expect.arrayContaining([
        expect.objectContaining({
          type: 'add',
        }),
      ])
    );
  });

  it('handles node selection', async () => {
    const mockNode = {
      id: 'node-1',
      type: 'trigger',
      position: { x: 100, y: 100 },
      data: { label: 'Test Node' },
    };

    render(
      <WorkflowCanvas
        {...defaultProps}
        nodes={[mockNode]}
      />
    );

    const nodeElement = screen.getByTestId('node-node-1');
    await userEvent.click(nodeElement);

    expect(nodeElement).toHaveClass('selected');
  });
});
```

#### Hook 测试
```typescript
// hooks/__tests__/useWorkflow.test.tsx
import { renderHook, act } from '@testing-library/react';
import { useWorkflow } from '../useWorkflow';
import { WorkflowProvider } from '../../context/WorkflowContext';

// Mock store
jest.mock('../../store/workflowStore');

const mockStore = {
  workflows: [],
  currentWorkflow: null,
  selectedNodes: [],
  setWorkflows: jest.fn(),
  setCurrentWorkflow: jest.fn(),
  addNode: jest.fn(),
  updateNode: jest.fn(),
  deleteNode: jest.fn(),
};

jest.mock('../../store/workflowStore', () => ({
  useWorkflowStore: () => mockStore,
}));

describe('useWorkflow', () => {
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <WorkflowProvider>{children}</WorkflowProvider>
  );

  it('returns workflow state', () => {
    const { result } = renderHook(() => useWorkflow(), { wrapper });

    expect(result.current.workflows).toEqual([]);
    expect(result.current.currentWorkflow).toBeNull();
    expect(result.current.selectedNodes).toEqual([]);
  });

  it('calls setWorkflows when calling setWorkflows', () => {
    const { result } = renderHook(() => useWorkflow(), { wrapper });

    const mockWorkflows = [
      { id: '1', name: 'Workflow 1' },
      { id: '2', name: 'Workflow 2' },
    ];

    act(() => {
      result.current.setWorkflows(mockWorkflows);
    });

    expect(mockStore.setWorkflows).toHaveBeenCalledWith(mockWorkflows);
  });
});
```

#### 集成测试
```typescript
// __tests__/integration/WorkflowExecution.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { WorkflowEditor } from '../WorkflowEditor';
import { mockApi } from '../__mocks__/api';

// Mock API
jest.mock('../services/api', () => mockApi);

describe('Workflow Execution Integration', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('executes complete workflow successfully', async () => {
    render(<WorkflowEditor />);

    // 创建工作流
    const addNodeButton = screen.getByTestId('add-node');
    await userEvent.click(addNodeButton);

    // 配置节点
    const nodeConfig = screen.getByTestId('node-config');
    await userEvent.type(nodeConfig, 'Test Configuration');

    // 执行工作流
    const executeButton = screen.getByTestId('execute-workflow');
    await userEvent.click(executeButton);

    // 验证执行结果
    await waitFor(() => {
      expect(screen.getByText('执行成功')).toBeInTheDocument();
    });

    // 验证 API 调用
    expect(mockApi.executeWorkflow).toHaveBeenCalled();
  });

  it('handles execution errors gracefully', async () => {
    // Mock API 错误
    mockApi.executeWorkflow.mockRejectedValueOnce(new Error('Execution failed'));

    render(<WorkflowEditor />);

    // 执行工作流
    const executeButton = screen.getByTestId('execute-workflow');
    await userEvent.click(executeButton);

    // 验证错误处理
    await waitFor(() => {
      expect(screen.getByText('执行失败')).toBeInTheDocument();
    });
  });
});
```

## 性能优化

### React 性能优化

#### 1. 组件优化
```typescript
// 使用 React.memo 避免不必要的重渲染
export const OptimizedComponent = React.memo<ComponentProps>(
  ({ data, onUpdate }) => {
    // 组件逻辑
    return <div>{data.value}</div>;
  },
  (prevProps, nextProps) => {
    // 自定义比较函数
    return prevProps.data.id === nextProps.data.id &&
           prevProps.data.value === nextProps.data.value;
  }
);

// 使用 useMemo 缓存计算结果
const ExpensiveComponent = ({ items }: { items: Item[] }) => {
  const expensiveValue = useMemo(() => {
    return items.reduce((sum, item) => sum + item.value, 0);
  }, [items]);

  return <div>Total: {expensiveValue}</div>;
};

// 使用 useCallback 缓存函数
const ComponentWithCallback = ({ onSubmit }: Props) => {
  const [value, setValue] = useState('');

  const handleSubmit = useCallback(() => {
    onSubmit(value);
  }, [value, onSubmit]);

  return (
    <div>
      <input value={value} onChange={(e) => setValue(e.target.value)} />
      <button onClick={handleSubmit}>Submit</button>
    </div>
  );
};
```

#### 2. 数据获取优化
```typescript
// 使用 React Query 优化数据获取
const useWorkflows = () => {
  return useQuery({
    queryKey: ['workflows'],
    queryFn: fetchWorkflows,
    staleTime: 5 * 60 * 1000, // 5 分钟缓存
    cacheTime: 10 * 60 * 1000, // 10 分钟存储
    refetchOnWindowFocus: false, // 窗口聚焦时不重新获取
    refetchOnReconnect: true, // 重新连接时重新获取
  });
};

// 使用分页加载大数据集
const useWorkflowsPaginated = () => {
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(true);

  const {
    data,
    isLoading,
    isFetching,
    fetchNextPage,
  } = useInfiniteQuery({
    queryKey: ['workflows', 'paginated'],
    queryFn: ({ pageParam = 1 }) => fetchWorkflows({ page: pageParam }),
    getNextPageParam: (lastPage, allPages) => {
      if (lastPage.length < 20) return undefined;
      return lastPage.length + 1;
    },
  });

  return {
    workflows: data?.pages.flat() ?? [],
    isLoading,
    isFetching,
    hasMore,
    fetchNextPage,
  };
};
```

#### 3. 虚拟化长列表
```typescript
// 使用 react-window 虚拟化长列表
import { FixedSizeList as List } from 'react-window';

const WorkflowNodeList = ({ nodes }: { nodes: WorkflowNode[] }) => {
  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => (
    <div style={style}>
      <WorkflowNode data={nodes[index]} />
    </div>
  );

  return (
    <List
      height={600}
      itemCount={nodes.length}
      itemSize={80}
      itemData={nodes}
    >
      {Row}
    </List>
  );
};
```

### 内存优化

#### 1. 组件卸载清理
```typescript
const WebSocketComponent = () => {
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    wsRef.current = new WebSocket('ws://localhost:3001');

    return () => {
      // 清理 WebSocket 连接
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, []);
};
```

#### 2. 大对象处理
```typescript
// 使用 WeakMap 避免内存泄漏
const memoCache = new WeakMap<object, any>();

export const memoize = <T extends (...args: any[]) => any>(
  fn: T,
  getKey?: (...args: Parameters<T>) => string
) => {
  return (...args: Parameters<T>) => {
    const key = getKey ? getKey(...args) : JSON.stringify(args);
    const cacheKey = { key, args } as object;

    if (memoCache.has(cacheKey)) {
      return memoCache.get(cacheKey);
    }

    const result = fn(...args);
    memoCache.set(cacheKey, result);
    return result;
  };
};
```

### 性能监控

#### 1. 性能指标收集
```typescript
// 性能监控 Hook
export const usePerformanceMonitor = () => {
  const [metrics, setMetrics] = useState({
    renderTime: 0,
    memoryUsage: 0,
    componentCount: 0,
  });

  const measureRender = useCallback(() => {
    const startTime = performance.now();

    return () => {
      const endTime = performance.now();
      const renderTime = endTime - startTime;

      setMetrics(prev => ({
        ...prev,
        renderTime,
      }));
    };
  }, []);

  const measureMemory = useCallback(() => {
    if ('memory' in performance) {
      const memory = (performance as any).memory;
      setMetrics(prev => ({
        ...prev,
        memoryUsage: memory.usedJSHeapSize / 1024 / 1024, // MB
      }));
    }
  }, []);

  return {
    metrics,
    measureRender,
    measureMemory,
  };
};
```

#### 2. 性能告警
```typescript
// 性能告警 Hook
export const usePerformanceAlert = (thresholds: {
  renderTime: number;
  memoryUsage: number;
}) => {
  const metrics = usePerformanceMonitor();
  const [alerts, setAlerts] = useState<string[]>([]);

  useEffect(() => {
    const newAlerts: string[] = [];

    if (metrics.renderTime > thresholds.renderTime) {
      newAlerts.push(
        `渲染时间过长: ${metrics.renderTime.toFixed(2)}ms`
      );
    }

    if (metrics.memoryUsage > thresholds.memoryUsage) {
      newAlerts.push(
        `内存使用过高: ${metrics.memoryUsage.toFixed(2)}MB`
      );
    }

    setAlerts(newAlerts);
  }, [metrics, thresholds]);

  return alerts;
};
```

## 部署指南

### 构建配置

#### Vite 配置
```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          antd: ['antd'],
          reactFlow: ['@xyflow/react'],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
  server: {
    port: 3000,
    host: true,
  },
});
```

#### 环境变量配置
```typescript
// vite-env.d.ts
interface ImportMetaEnv {
  readonly VITE_APP_API_URL: string;
  readonly VITE_APP_WS_URL: string;
  readonly VITE_APP_ENV: string;
  readonly VITE_APP_VERSION: string;
}

export interface ImportMeta {
  readonly env: ImportMetaEnv;
}
```

### Docker 部署

#### Dockerfile
```dockerfile
# 多阶段构建
FROM node:22-alpine AS builder

WORKDIR /app

# 复制 package 文件
COPY package*.json ./
COPY pnpm-lock.yaml ./

# 安装依赖
RUN npm install -g pnpm
RUN pnpm install --frozen-lockfile

# 复制源代码
COPY . .

# 构建应用
RUN pnpm build

# 生产镜像
FROM nginx:alpine

# 复制构建产物
COPY --from=builder /app/dist /usr/share/nginx/html

# 复制配置文件
COPY nginx.conf /etc/nginx/nginx.conf

# 设置权限
RUN chown -R nginx:nginx /usr/share/nginx/html && \
    chmod -R 755 /usr/share/nginx/html

# 暴露端口
EXPOSE 80

# 启动 nginx
CMD ["nginx", "-g", "daemon off;"]
```

#### Nginx 配置
```nginx
# nginx.conf
events {
    worker_connections 1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    # Gzip 压缩
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/javascript
        application/xml+rss
        application/json;

    server {
        listen 80;
        server_name localhost;
        root /usr/share/nginx/html;
        index index.html;

        # 静态资源缓存
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }

        # 路由配置
        location / {
            try_files $uri $uri/ /index.html;
        }

        # API 代理
        location /api/ {
            proxy_pass http://backend:3001;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }
    }
}
```

#### Docker Compose
```yaml
# docker-compose.yml
version: '3.8'

services:
  frontend:
    build: .
    ports:
      - "80:80"
    environment:
      - NODE_ENV=production
    restart: unless-stopped

  backend:
    image: hetumind/backend:latest
    ports:
      - "3001:3001"
    environment:
      - DATABASE_URL=postgresql://user:password@db:5432/hetumind
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=hetumind
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
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
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '22'
          cache: 'pnpm'

      - name: Install dependencies
        run: pnpm install --frozen-lockfile

      - name: Run tests
        run: pnpm test:ci

      - name: Run type check
        run: pnpm type-check

      - name: Run linting
        run: pnpm lint

      - name: Upload coverage reports
        uses: codecov/codecov-action@v3
        with:
          file: ./coverage/lcov.info

  build:
    needs: test
    runs-on: ubuntu-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '22'
          cache: 'pnpm'

      - name: Install dependencies
        run: pnpm install --frozen-lockfile

      - name: Build application
        run: pnpm build

      - name: Build Docker image
        run: |
          docker build -t hetumind/web:${{ github.sha }} .
          docker tag hetumind/web:latest .

      - name: Push to registry
        run: |
          echo ${{ secrets.DOCKER_PASSWORD }} | docker login -u ${{ secrets.DOCKER_USERNAME }} --password-stdin
          docker push hetumind/web:${{ github.sha }}
          docker push hetumind/web:latest

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment: production

    steps:
      - name: Deploy to production
        run: |
          # 部署脚本
          echo "Deploying to production..."
```

## 贡献流程

### 开发流程

#### 1. Fork 项目
```bash
# Fork 项目到个人仓库
git clone https://github.com/your-username/hetumind.git
cd hetumind
git remote add upstream https://github.com/fusion-data/hetumind.git
```

#### 2. 创建功能分支
```bash
# 创建并切换到功能分支
git checkout -b feature/amazing-feature

# 推送到个人仓库
git push origin feature/amazing-feature
```

#### 3. 开发和测试
```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev

# 运行测试
pnpm test

# 类型检查
pnpm type-check

# 代码格式化
pnpm lint:fix
```

#### 4. 提交更改
```bash
# 添加所有更改
git add .

# 提交更改
git commit -m "feat: add amazing feature"

# 推送到个人仓库
git push origin feature/amazing-feature
```

#### 5. 创建 Pull Request
- 在 GitHub 上创建 Pull Request
- 填写清晰的 PR 描述
- 等待代码审查
- 根据反馈进行修改

### 代码审查

#### 审查清单
- [ ] 代码符合项目规范
- [ ] 类型检查通过
- [ ] 测试覆盖率达标
- [ ] 性能影响可接受
- [ ] 文档已更新

#### 审查要点
- **代码质量**: 可读性、可维护性
- **功能正确性**: 实现符合需求
- **性能影响**: 是否存在性能问题
- **安全性**: 是否存在安全漏洞
- **测试覆盖**: 是否有充分的测试

### 发布流程

#### 1. 合并到主分支
- 所有 CI 检查通过
- 代码审查完成
- 冲突已解决

#### 2. 自动部署
- 自动触发构建和部署
- 部署到测试环境进行验证
- 部署到生产环境

#### 3. 版本发布
- 创建版本标签
- 生成发布说明
- 更新文档

### 版本管理

#### 语义化版本
```
主版本号.次版本号.修订号

例如: 1.2.3

- 主版本号: 不兼容的 API 修改
- 次版本号: 向下兼容的功能性新增
- 修订号: 向下兼容的问题修正
```

#### 发布流程
```bash
# 创建版本标签
git tag -a v1.2.3 -m "Release version 1.2.3"

# 推送标签
git push origin v1.2.3

# 创建发布分支
git checkout -b release/v1.2.3

# 更新版本号
npm version 1.2.3

# 提交版本更新
git commit -m "chore(release): bump version to 1.2.3"

# 推送版本分支
git push origin release/v1.2.3

# 创建 Pull Request 合并到主分支
```

---

通过遵循这些开发指南和最佳实践，我们可以构建一个高质量、可维护的 Hetumind Web 平台。

如有任何问题或建议，请查看项目文档或联系开发团队。