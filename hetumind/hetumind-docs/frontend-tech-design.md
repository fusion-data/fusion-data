# Hetumind Web 前端技术设计方案

## 1. 项目概述

### 1.1 项目背景

**Hetumind Web** 是 Hetumind Studio 的前端项目，为 AI Agent 开发和工作流编排提供可视化的 Web 界面。作为 fusion-data 平台的重要组成部分，Hetumind Web 将为用户提供直观的拖拽式工作流设计、AI Agent 配置、数据映射和实时监控等功能。

### 1.2 设计目标

- **可视化工作流编排**: 提供类似 n8n 的节点式工作流编辑器
- **AI Agent 开发平台**: 支持可视化配置和管理 AI 智能体
- **现代化技术栈**: 基于 React 19 + TypeScript + Ant Design 5
- **主题系统**: 支持亮色/暗色主题切换
- **模块化架构**: 清晰的组件分层和功能模块划分

## 2. 技术架构设计

### 2.1 技术栈选型

#### 核心技术栈

```typescript
// 技术栈配置
{
  "前端框架": "React 19.1.1",
  "开发语言": "TypeScript 5.9.2",
  "构建工具": "Vite 7.1.7",
  "UI 框架": "Ant Design 5.27.4",
  "路由管理": "React Router DOM 7.9.3",
  "状态管理": "Zustand (轻量级状态管理)",
  "拖拽系统": "@dnd-kit/core + @dnd-kit/sortable",
  "可视化引擎": "React Flow (工作流画布)",
  // 迁移：使用 React Flow 12（@xyflow/react）
  // 详见迁移指南 https://reactflow.dev/learn/troubleshooting/migrate-to-v12
  "可视化引擎": "React Flow 12 (@xyflow/react)",
  "代码编辑器": "@monaco-editor/react",
  "样式方案": "CSS Modules + CSS Variables",
  "开发工具": "ESLint + Prettier + TypeScript",
  "数据获取与缓存": "@tanstack/react-query"
}
```

#### 项目依赖结构

```json
{
  "dependencies": {
    "@ant-design/icons": "^6.0.2",
    "@ant-design/pro-components": "^2.8.10",
    "@ant-design/v5-patch-for-react-19": "^1.0.3",
    "@dnd-kit/core": "^6.3.1",
    "@dnd-kit/sortable": "^10.0.0",
    "@dnd-kit/utilities": "^3.2.2",
    "@monaco-editor/react": "^4.6.0",
    "@tanstack/react-query": "^5.62.3",
    "@fusion-data/fusion-core": "workspace:*",
    "@fusion-data/fusionsql": "workspace:*",
    "react": "^19.1.1",
    "react-dom": "^19.1.1",
    "react-router-dom": "^7.9.3",
    "@xyflow/react": "^12.8.6",
    "zustand": "^5.0.2",
    "immer": "^10.1.1",
    "uuid": "^13.0.0",
    "dayjs": "^1.11.18",
    "antd": "^5.27.4"
  }
}
```

### 2.2 项目结构设计

```
hetumind-web/
├── src/
│   ├── components/           # 通用组件
│   │   ├── layout/          # 布局组件
│   │   │   ├── MainLayout.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── Header.tsx
│   │   ├── workflow/        # 工作流相关组件
│   │   │   ├── Canvas/
│   │   │   │   ├── WorkflowCanvas.tsx
│   │   │   │   ├── NodeTypes/
│   │   │   │   ├── EdgeTypes/
│   │   │   │   └── Controls/
│   │   │   ├── NodePanel/
│   │   │   │   ├── NodeLibrary.tsx
│   │   │   │   ├── NodeSearch.tsx
│   │   │   │   └── NodeCategories.tsx
│   │   │   └── PropertyPanel/
│   │   │       ├── NodeProperties.tsx
│   │   │       └── DataMapping.tsx
│   │   ├── agent/           # AI Agent 相关组件
│   │   │   ├── AgentEditor.tsx
│   │   │   ├── AgentConfig.tsx
│   │   │   └── PromptEditor.tsx
│   │   ├── ui/              # 基础 UI 组件
│   │   │   ├── ThemeSwitcher.tsx
│   │   │   ├── ColorMode.tsx
│   │   │   └── Loading.tsx
│   │   └── common/          # 通用业务组件
│   │       ├── CodeEditor.tsx
│   │       ├── DataViewer.tsx
│   │       └── DragDropContext.tsx
│   ├── pages/               # 页面组件
│   │   ├── Dashboard/
│   │   │   └── index.tsx
│   │   ├── Workflows/
│   │   │   ├── index.tsx
│   │   │   ├── Editor.tsx
│   │   │   └── Detail.tsx
│   │   ├── Agents/
│   │   │   ├── index.tsx
│   │   │   ├── Editor.tsx
│   │   │   └── List.tsx
│   │   ├── Settings/
│   │   │   └── index.tsx
│   │   └── Login/
│   │       └── index.tsx
│   ├── hooks/               # 自定义 Hooks
│   │   ├── useTheme.ts
│   │   ├── useWorkflow.ts
│   │   ├── useAgent.ts
│   │   ├── useDragDrop.ts
│   │   └── useLocalStorage.ts
│   ├── stores/              # 状态管理 (Zustand)
│   │   ├── workflowStore.ts
│   │   ├── agentStore.ts
│   │   ├── themeStore.ts
│   │   └── appStore.ts
│   ├── contexts/            # React Context
│   │   └── ThemeContext.tsx
│   ├── types/               # TypeScript 类型定义
│   │   ├── workflow.ts
│   │   ├── agent.ts
│   │   ├── node.ts
│   │   └── api.ts
│   ├── utils/               # 工具函数
│   │   ├── workflow.ts
│   │   ├── node.ts
│   │   ├── validation.ts
│   │   └── storage.ts
│   ├── services/            # API 服务
│   │   └── api.ts
│   ├── styles/              # 样式文件
│   │   ├── globals.css
│   │   ├── variables.css
│   │   └── themes.css
│   └── assets/              # 静态资源
│       ├── icons/
│       └── images/
├── public/                  # 公共资源
├── docs/                    # 项目文档
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

### 2.3 核心架构模式

#### MVVM 架构

```typescript
// Model - 数据模型层
interface WorkflowModel {
  id: string;
  name: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  metadata: WorkflowMetadata;
}

// ViewModel - 视图模型层 (Zustand Store)
interface WorkflowStore {
  // State
  workflows: WorkflowModel[];
  activeWorkflow: WorkflowModel | null;
  selectedNodes: string[];

  // Actions
  createWorkflow: (name: string) => void;
  addNode: (node: WorkflowNode) => void;
  removeNode: (nodeId: string) => void;
  updateNode: (nodeId: string, updates: Partial<WorkflowNode>) => void;
  createConnection: (edge: WorkflowEdge) => void;
}

// View - 视图层 (React Components)
const WorkflowEditor: React.FC = () => {
  const { activeWorkflow, addNode, removeNode } = useWorkflowStore();

  return (
    <div className="workflow-editor">
      <WorkflowCanvas />
      <NodePanel onAddNode={addNode} />
      <PropertyPanel />
    </div>
  );
};
```

#### 节点插件化与动态加载

- 节点注册中心：定义节点包 manifest（元数据 + schema + 渲染器入口），在前端完成节点的注册与分类展示。
- 动态加载与沙箱：第三方/社区节点渲染器采用 `dynamic import` 按需加载，敏感或外部节点在受限沙箱中渲染（执行逻辑在后端）。

## 3. 工作流可视化编排系统

### 3.1 基于 React Flow 的画布系统

#### 画布核心组件架构

```typescript
// WorkflowCanvas.tsx
import React, { useCallback, useMemo } from 'react';
import {
  ReactFlow,
  Node,
  Edge,
  addEdge,
  useNodesState,
  useEdgesState,
  Controls,
  MiniMap,
  Background,
  Connection,
  NodeChange,
  EdgeChange,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

interface WorkflowCanvasProps {
  workflowId: string;
  onNodeChange?: (nodes: Node[]) => void;
  onEdgeChange?: (edges: Edge[]) => void;
}

const WorkflowCanvas: React.FC<WorkflowCanvasProps> = ({ workflowId, onNodeChange, onEdgeChange }) => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  // 自定义节点类型
  const nodeTypes = useMemo(
    () => ({
      aiAgent: AIAgentNode,
      dataProcessor: DataProcessorNode,
      trigger: TriggerNode,
      condition: ConditionNode,
      webhook: WebhookNode,
    }),
    []
  );

  // 自定义连接线类型
  const edgeTypes = useMemo(
    () => ({
      smartEdge: SmartEdge,
      animatedEdge: AnimatedEdge,
    }),
    []
  );

  // 连接处理
  const onConnect = useCallback(
    (params: Connection) => {
      const newEdge = {
        ...params,
        id: `edge-${Date.now()}`,
        type: 'smartEdge',
        animated: true,
      };
      setEdges(eds => addEdge(newEdge, eds));
    },
    [setEdges]
  );

  return (
    <div className="workflow-canvas">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        fitView
        attributionPosition="bottom-left"
      >
        <Controls />
        <MiniMap />
        <Background variant="dots" gap={12} size={1} />
      </ReactFlow>
    </div>
  );
};
```

// 说明：为大规模工作流的交互体验，编辑器集成了“数据 Pin/回放”与“节点日志面板”，并在低缩放级别自动降级边线渲染与动画，以提升性能与可读性。

#### 节点类型系统

```typescript
// types/node.ts
export interface BaseNodeData {
  id: string;
  type: string;
  name: string;
  description?: string;
  inputs: NodePort[];
  outputs: NodePort[];
  config: Record<string, any>;
  status: NodeStatus;
  position: { x: number; y: number };
}

export interface AIAgentNodeData extends BaseNodeData {
  type: 'aiAgent';
  agentType: 'chat' | 'completion' | 'embedding' | 'image';
  model: string;
  prompt: string;
  parameters: {
    temperature?: number;
    maxTokens?: number;
    topP?: number;
    frequencyPenalty?: number;
    presencePenalty?: number;
  };
}

export interface DataProcessorNodeData extends BaseNodeData {
  type: 'dataProcessor';
  processorType: 'mapper' | 'filter' | 'aggregator' | 'transformer';
  mappingRules: MappingRule[];
  filterConditions: FilterCondition[];
}

// 节点渲染器
const AIAgentNode: React.FC<NodeProps> = ({ data, selected }) => {
  const { updateNodeConfig } = useWorkflowStore();

  return (
    <div className={`ai-agent-node ${selected ? 'selected' : ''}`}>
      <NodeHeader icon={<RobotOutlined />} title={data.name} type={data.agentType} status={data.status} />
      <NodeContent>
        <div className="agent-info">
          <Tag color="blue">{data.model}</Tag>
          <Text type="secondary" ellipsis>
            {data.prompt.substring(0, 50)}...
          </Text>
        </div>
      </NodeContent>
      <NodeHandles inputs={data.inputs} outputs={data.outputs} />
    </div>
  );
};
```

### 3.2 拖拽节点系统

#### 节点面板组件（统一使用 dnd-kit）

```typescript
// NodePanel/NodeLibrary.tsx
import React, { useState } from 'react';
import { DndContext, useDraggable } from '@dnd-kit/core';
import { Card, Radio, Space, Typography } from 'antd';
import { RobotOutlined, DatabaseOutlined } from '@ant-design/icons';
const { Text } = Typography;

interface NodeTypeItem {
  type: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  category: string;
}

const nodeTypes: NodeTypeItem[] = [
  {
    type: 'aiAgent',
    name: 'AI 智能体',
    description: '大语言模型节点，支持对话和文本生成',
    icon: <RobotOutlined />,
    category: 'AI',
  },
  {
    type: 'dataProcessor',
    name: '数据处理',
    description: '数据转换、过滤和聚合处理',
    icon: <DatabaseOutlined />,
    category: 'Data',
  },
  // ... 更多节点类型
];

/* 函数级注释：侧边栏节点项（dnd-kit 拖拽源） */
const DraggableNode: React.FC<{ nodeType: NodeTypeItem }> = ({ nodeType }) => {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: `node-${nodeType.type}`,
    data: { nodeType },
  });
  return (
    <div ref={setNodeRef} {...attributes} {...listeners} className={`draggable-node ${isDragging ? 'dragging' : ''}`}>
      <Card size="small" hoverable>
        <Space>
          {nodeType.icon}
          <div>
            <div className="node-name">{nodeType.name}</div>
            <div className="node-description">
              <Text type="secondary">{nodeType.description}</Text>
            </div>
          </div>
        </Space>
      </Card>
    </div>
  );
};

const NodeLibrary: React.FC = () => {
  const [selectedCategory, setSelectedCategory] = useState<string>('all');

  const categories = [
    { key: 'all', name: '全部' },
    { key: 'AI', name: 'AI 智能体' },
    { key: 'Data', name: '数据处理' },
    { key: 'Trigger', name: '触发器' },
    { key: 'Control', name: '控制流' },
  ];

  const filteredNodes =
    selectedCategory === 'all' ? nodeTypes : nodeTypes.filter(node => node.category === selectedCategory);

  return (
    <DndContext>
      <div className="node-library" data-testid="node-library">
        <div className="node-categories">
          <Radio.Group
            value={selectedCategory}
            onChange={e => setSelectedCategory(e.target.value)}
            buttonStyle="solid"
            size="small"
          >
            {categories.map(category => (
              <Radio.Button key={category.key} value={category.key}>
                {category.name}
              </Radio.Button>
            ))}
          </Radio.Group>
        </div>

        <div className="node-list">
          {filteredNodes.map(nodeType => (
            <DraggableNode key={nodeType.type} nodeType={nodeType} />
          ))}
        </div>
      </div>
    </DndContext>
  );
};
```

#### 画布拖拽接收

```typescript
// Canvas/DropZone.tsx
import { DndContext, useDroppable } from '@dnd-kit/core';

const CanvasDropZone: React.FC = () => {
  const { addNode } = useWorkflowStore();
  const { setNodeRef, isOver } = useDroppable({ id: 'canvas-drop-zone' });

  /* 函数级注释：处理拖拽结束，在画布创建节点 */
  const handleDragEnd = (event: any) => {
    const nodeType: NodeTypeItem | undefined = event.active?.data?.current?.nodeType;
    const overId: string | undefined = event.over?.id;
    if (!nodeType || overId !== 'canvas-drop-zone') return;

    const position = { x: 120, y: 80 }; // TODO: 根据指针位置或画布坐标系计算

    const newNode: WorkflowNode = {
      id: generateId(),
      type: nodeType.type,
      name: nodeType.name,
      position,
      data: getDefaultNodeData(nodeType.type),
    };
    addNode(newNode);
  };

  return (
    <DndContext onDragEnd={handleDragEnd}>
      <div ref={setNodeRef} className={`canvas-drop-zone ${isOver ? 'drop-active' : ''}`}>
        {/* Workflow Canvas 内容 */}
      </div>
    </DndContext>
  );
};
```

#### 坐标映射与 DragOverlay 示例（dnd-kit + React Flow）

```typescript
// Canvas/DropZoneWithOverlay.tsx
import { DndContext, DragOverlay, useDroppable } from '@dnd-kit/core';
import { useRef, useState } from 'react';
import { useReactFlow } from '@xyflow/react';

/* 函数级注释：通过 DragOverlay 与 React Flow 的 project 完成屏幕坐标到画布坐标映射 */
const CanvasDropZoneWithOverlay: React.FC = () => {
  const { addNode } = useWorkflowStore();
  const { setNodeRef, isOver } = useDroppable({ id: 'canvas-drop-zone' });
  const canvasRef = useRef<HTMLDivElement | null>(null);
  const rf = useReactFlow();
  const [dragPreview, setDragPreview] = useState<{ name: string } | null>(null);
  const pointerRef = useRef<{ x: number; y: number }>({ x: 0, y: 0 });

  const handleDragMove = (event: any) => {
    // 记录指针位置（MouseEvent/Touchevent），用于计算落点
    const e = (event.activatorEvent || event?.deltaEvent) as MouseEvent | PointerEvent | TouchEvent | undefined;
    const clientX = (e as any)?.clientX ?? pointerRef.current.x;
    const clientY = (e as any)?.clientY ?? pointerRef.current.y;
    pointerRef.current = { x: clientX, y: clientY };

    // 可选：更新拖拽预览内容
    const nodeType: NodeTypeItem | undefined = event.active?.data?.current?.nodeType;
    setDragPreview(nodeType ? { name: nodeType.name } : null);
  };

  const handleDragEnd = (event: any) => {
    const nodeType: NodeTypeItem | undefined = event.active?.data?.current?.nodeType;
    const overId: string | undefined = event.over?.id;
    if (!nodeType || overId !== 'canvas-drop-zone') return;

    const bounds = canvasRef.current?.getBoundingClientRect();
    if (!bounds) return;

    // 将屏幕坐标映射到画布坐标（考虑 React Flow 的缩放/平移）
    const local = {
      x: pointerRef.current.x - bounds.left,
      y: pointerRef.current.y - bounds.top,
    };
    const position = rf.project(local);

    const newNode: WorkflowNode = {
      id: generateId(),
      type: nodeType.type,
      name: nodeType.name,
      position,
      data: getDefaultNodeData(nodeType.type),
    };
    addNode(newNode);
    setDragPreview(null);
  };

  return (
    <DndContext onDragMove={handleDragMove} onDragEnd={handleDragEnd}>
      <div
        ref={el => {
          setNodeRef(el);
          canvasRef.current = el;
        }}
        className={`canvas-drop-zone ${isOver ? 'drop-active' : ''}`}
      >
        {/* Workflow Canvas 内容 */}
      </div>

      {/* 拖拽预览（可按需美化） */}
      <DragOverlay>{dragPreview ? <div className="drag-overlay">拖拽：{dragPreview.name}</div> : null}</DragOverlay>
    </DndContext>
  );
};
```

### 3.3 属性面板系统

#### 节点属性编辑器

```typescript
// PropertyPanel/NodeProperties.tsx
import React from 'react';
import { Form, Input, Select, Switch, Slider, Button } from 'antd';

interface NodePropertiesProps {
  nodeId: string;
}

const NodeProperties: React.FC<NodePropertiesProps> = ({ nodeId }) => {
  const { getNode, updateNode } = useWorkflowStore();
  const [form] = Form.useForm();

  const node = getNode(nodeId);

  const handleFormChange = (changedValues: any, allValues: any) => {
    updateNode(nodeId, { data: { ...node.data, ...allValues } });
  };

  const renderPropertyFields = () => {
    switch (node.type) {
      case 'aiAgent':
        return (
          <>
            <Form.Item label="智能体类型" name="agentType">
              <Select>
                <Select.Option value="chat">对话</Select.Option>
                <Select.Option value="completion">文本生成</Select.Option>
                <Select.Option value="embedding">文本嵌入</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item label="模型" name="model">
              <Select>
                <Select.Option value="gpt-4">GPT-4</Select.Option>
                <Select.Option value="gpt-3.5-turbo">GPT-3.5 Turbo</Select.Option>
                <Select.Option value="claude-3">Claude 3</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item label="提示词" name="prompt">
              <Input.TextArea rows={4} placeholder="输入 AI 提示词..." />
            </Form.Item>

            <Form.Item label="温度" name={['parameters', 'temperature']}>
              <Slider min={0} max={2} step={0.1} />
            </Form.Item>

            <Form.Item label="最大令牌数" name={['parameters', 'maxTokens']}>
              <Input type="number" />
            </Form.Item>
          </>
        );

      case 'dataProcessor':
        return (
          <>
            <Form.Item label="处理器类型" name="processorType">
              <Select>
                <Select.Option value="mapper">数据映射</Select.Option>
                <Select.Option value="filter">数据过滤</Select.Option>
                <Select.Option value="aggregator">数据聚合</Select.Option>
              </Select>
            </Form.Item>

            {/* 数据处理特定配置 */}
          </>
        );

      default:
        return null;
    }
  };

  return (
    <div className="node-properties">
      <div className="properties-header">
        <h3>{node.name}</h3>
        <Text type="secondary">{node.type}</Text>
      </div>

      <Form form={form} layout="vertical" initialValues={node.data} onValuesChange={handleFormChange}>
        {renderPropertyFields()}
      </Form>
    </div>
  );
};
```

### 3.4 调试与回放系统（新增）

- 数据 Pin：在节点上固定样例输入，画布内快速调试节点行为，无需外部触发。
- 执行历史回放：从执行历史选择一次执行作为回放源，将节点输入/输出重放到画布，结合表达式预览与日志面板定位问题。
- 节点日志面板：展示节点级输入/输出、耗时、重试次数、异常摘要与错误栈（若可用）。

## 4. 主题系统设计

### 4.1 主题管理架构

#### ThemeContext 设计

```typescript
// contexts/ThemeContext.tsx
import React, { createContext, useContext, useEffect, useState } from 'react';
import { theme as antTheme } from 'antd';

export type ThemeMode = 'light' | 'dark' | 'system';
export type ColorScheme = 'blue' | 'purple' | 'green' | 'orange';

interface ThemeContextType {
  themeMode: ThemeMode;
  colorScheme: ColorScheme;
  setThemeMode: (mode: ThemeMode) => void;
  setColorScheme: (scheme: ColorScheme) => void;
  currentTheme: 'light' | 'dark';
  antdTheme: any;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [themeMode, setThemeMode] = useState<ThemeMode>('system');
  const [colorScheme, setColorScheme] = useState<ColorScheme>('blue');
  const [currentTheme, setCurrentTheme] = useState<'light' | 'dark'>('light');

  // 获取系统主题
  const getSystemTheme = (): 'light' | 'dark' => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  };

  // 计算当前主题
  useEffect(() => {
    const theme = themeMode === 'system' ? getSystemTheme() : themeMode;
    setCurrentTheme(theme);
    applyTheme(theme, colorScheme);
  }, [themeMode, colorScheme]);

  // 监听系统主题变化
  useEffect(() => {
    if (themeMode !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (e: MediaQueryListEvent) => {
      setCurrentTheme(e.matches ? 'dark' : 'light');
      applyTheme(e.matches ? 'dark' : 'light', colorScheme);
    };

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [themeMode, colorScheme]);

  // 应用主题到 DOM
  const applyTheme = (theme: 'light' | 'dark', scheme: ColorScheme) => {
    const root = document.documentElement;
    root.setAttribute('data-theme', theme);
    root.setAttribute('data-color-scheme', scheme);

    // 应用 CSS 变量
    const colors = getThemeColors(theme, scheme);
    Object.entries(colors).forEach(([key, value]) => {
      root.style.setProperty(`--${key}`, value);
    });
  };

  // Ant Design 主题配置
  const antdTheme = React.useMemo(() => {
    const baseColors = {
      blue: { primary: '#1890ff', success: '#52c41a', warning: '#faad14', error: '#ff4d4f' },
      purple: { primary: '#722ed1', success: '#52c41a', warning: '#faad14', error: '#ff4d4f' },
      green: { primary: '#52c41a', success: '#73d13d', warning: '#faad14', error: '#ff4d4f' },
      orange: { primary: '#fa8c16', success: '#52c41a', warning: '#faad14', error: '#ff4d4f' },
    };

    const colors = baseColors[colorScheme];
    const algorithm = currentTheme === 'dark' ? antTheme.darkAlgorithm : antTheme.defaultAlgorithm;

    return {
      algorithm,
      token: {
        colorPrimary: colors.primary,
        colorSuccess: colors.success,
        colorWarning: colors.warning,
        colorError: colors.error,
        borderRadius: 8,
        wireframe: false,
      },
      components: {
        Layout: {
          siderBg: currentTheme === 'dark' ? '#001529' : '#fff',
          triggerBg: currentTheme === 'dark' ? '#002140' : '#f0f2f5',
        },
        Menu: {
          darkItemBg: '#001529',
          darkSubMenuItemBg: '#000c17',
          darkItemSelectedBg: colors.primary,
        },
      },
    };
  }, [currentTheme, colorScheme]);

  return (
    <ThemeContext.Provider
      value={{
        themeMode,
        colorScheme,
        setThemeMode,
        setColorScheme,
        currentTheme,
        antdTheme,
      }}
    >
      {children}
    </ThemeContext.Provider>
  );
};
```

#### 主题切换组件

```typescript
// components/ui/ThemeSwitcher.tsx
import React from 'react';
import { Button, Dropdown, Space, Tooltip } from 'antd';
import { SunOutlined, MoonOutlined, LaptopOutlined, BgColorsOutlined } from '@ant-design/icons';

const ThemeSwitcher: React.FC = () => {
  const { themeMode, colorScheme, setThemeMode, setColorScheme } = useTheme();

  const themeOptions = [
    { key: 'light', label: '浅色', icon: <SunOutlined /> },
    { key: 'dark', label: '深色', icon: <MoonOutlined /> },
    { key: 'system', label: '跟随系统', icon: <LaptopOutlined /> },
  ];

  const colorOptions = [
    { key: 'blue', label: '蓝色', color: '#1890ff' },
    { key: 'purple', label: '紫色', color: '#722ed1' },
    { key: 'green', label: '绿色', color: '#52c41a' },
    { key: 'orange', label: '橙色', color: '#fa8c16' },
  ];

  const currentThemeIcon = {
    light: <SunOutlined />,
    dark: <MoonOutlined />,
    system: <LaptopOutlined />,
  };

  return (
    <Space>
      <Dropdown
        menu={{
          items: themeOptions.map(option => ({
            key: option.key,
            label: option.label,
            icon: option.icon,
            onClick: () => setThemeMode(option.key as ThemeMode),
          })),
          selectedKeys: [themeMode],
        }}
        trigger={['click']}
      >
        <Tooltip title="主题模式">
          <Button type="text" icon={currentThemeIcon[themeMode]} />
        </Tooltip>
      </Dropdown>

      <Dropdown
        menu={{
          items: colorOptions.map(option => ({
            key: option.key,
            label: (
              <Space>
                <div
                  style={{
                    width: 12,
                    height: 12,
                    borderRadius: '50%',
                    backgroundColor: option.color,
                  }}
                />
                {option.label}
              </Space>
            ),
            onClick: () => setColorScheme(option.key as ColorScheme),
          })),
          selectedKeys: [colorScheme],
        }}
        trigger={['click']}
      >
        <Tooltip title="主题颜色">
          <Button type="text" icon={<BgColorsOutlined />} />
        </Tooltip>
      </Dropdown>
    </Space>
  );
};
```

### 4.2 CSS 主题变量

```css
/* styles/themes.css */
:root {
  /* 基础颜色变量 - 浅色主题 */
  --bg-primary: #ffffff;
  --bg-secondary: #f5f5f5;
  --bg-tertiary: #fafafa;
  --bg-canvas: #fafafa;

  --text-primary: #000000;
  --text-secondary: rgba(0, 0, 0, 0.85);
  --text-tertiary: rgba(0, 0, 0, 0.45);
  --text-disabled: rgba(0, 0, 0, 0.25);

  --border-primary: #d9d9d9;
  --border-secondary: #f0f0f0;
  --border-canvas: #e8e8e8;

  --shadow-1: 0 2px 8px rgba(0, 0, 0, 0.15);
  --shadow-2: 0 4px 12px rgba(0, 0, 0, 0.15);
  --shadow-3: 0 6px 16px rgba(0, 0, 0, 0.15);

  /* 工作流画布颜色 */
  --canvas-bg: #fafafa;
  --canvas-grid: #e8e8e8;
  --node-bg: #ffffff;
  --node-border: #d9d9d9;
  --node-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  --connection-stroke: #1890ff;
  --connection-stroke-selected: #ff7875;
}

/* 深色主题 */
[data-theme='dark'] {
  --bg-primary: #141414;
  --bg-secondary: #000000;
  --bg-tertiary: #1f1f1f;
  --bg-canvas: #000000;

  --text-primary: #ffffff;
  --text-secondary: rgba(255, 255, 255, 0.85);
  --text-tertiary: rgba(255, 255, 255, 0.65);
  --text-disabled: rgba(255, 255, 255, 0.45);

  --border-primary: #303030;
  --border-secondary: #434343;
  --border-canvas: #303030;

  --shadow-1: 0 2px 8px rgba(0, 0, 0, 0.45);
  --shadow-2: 0 4px 12px rgba(0, 0, 0, 0.45);
  --shadow-3: 0 6px 16px rgba(0, 0, 0, 0.45);

  /* 工作流画布颜色 */
  --canvas-bg: #000000;
  --canvas-grid: #303030;
  --node-bg: #1f1f1f;
  --node-border: #434343;
  --node-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  --connection-stroke: #1890ff;
  --connection-stroke-selected: #ff7875;
}

/* 主题颜色方案 */
[data-color-scheme='blue'] {
  --color-primary: #1890ff;
  --color-primary-hover: #40a9ff;
  --color-primary-active: #096dd9;
}

[data-color-scheme='purple'] {
  --color-primary: #722ed1;
  --color-primary-hover: #9254de;
  --color-primary-active: #531dab;
}

[data-color-scheme='green'] {
  --color-primary: #52c41a;
  --color-primary-hover: #73d13d;
  --color-primary-active: #389e0d;
}

[data-color-scheme='orange'] {
  --color-primary: #fa8c16;
  --color-primary-hover: #ffa940;
  --color-primary-active: #d46b08;
}
```

## 5. 状态管理设计

### 5.1 Zustand Store 架构

#### 工作流状态管理

```typescript
// stores/workflowStore.ts
import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { devtools } from 'zustand/middleware';

interface WorkflowState {
  // 工作流列表
  workflows: Workflow[];
  currentWorkflowId: string | null;

  // 节点状态
  nodes: Node[];
  edges: Edge[];
  selectedNodes: string[];
  selectedEdges: string[];

  // 画布状态
  viewport: Viewport;
  isCanvasReady: boolean;

  // Actions
  createWorkflow: (name: string) => void;
  deleteWorkflow: (id: string) => void;
  loadWorkflow: (id: string) => void;
  saveWorkflow: () => Promise<void>;

  // 节点操作
  addNode: (node: Node) => void;
  updateNode: (id: string, updates: Partial<Node>) => void;
  deleteNode: (id: string) => void;
  duplicateNode: (id: string) => void;

  // 连接操作
  addEdge: (edge: Edge) => void;
  updateEdge: (id: string, updates: Partial<Edge>) => void;
  deleteEdge: (id: string) => void;

  // 选择操作
  selectNodes: (nodeIds: string[]) => void;
  selectEdges: (edgeIds: string[]) => void;
  clearSelection: () => void;

  // 画布操作
  setViewport: (viewport: Viewport) => void;
  fitView: () => void;
  centerCanvas: () => void;
}

export const useWorkflowStore = create<WorkflowState>()(
  devtools(
    immer((set, get) => ({
      workflows: [],
      currentWorkflowId: null,
      nodes: [],
      edges: [],
      selectedNodes: [],
      selectedEdges: [],
      viewport: { x: 0, y: 0, zoom: 1 },
      isCanvasReady: false,

      createWorkflow: (name: string) => {
        const newWorkflow: Workflow = {
          id: generateId(),
          name,
          nodes: [],
          edges: [],
          metadata: {
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            version: '1.0.0',
          },
        };

        set(state => {
          state.workflows.push(newWorkflow);
          state.currentWorkflowId = newWorkflow.id;
          state.nodes = [];
          state.edges = [];
          state.selectedNodes = [];
          state.selectedEdges = [];
        });
      },

      addNode: (node: Node) => {
        set(state => {
          state.nodes.push(node);
        });
      },

      updateNode: (id: string, updates: Partial<Node>) => {
        set(state => {
          const nodeIndex = state.nodes.findIndex(node => node.id === id);
          if (nodeIndex !== -1) {
            Object.assign(state.nodes[nodeIndex], updates);
          }
        });
      },

      deleteNode: (id: string) => {
        set(state => {
          state.nodes = state.nodes.filter(node => node.id !== id);
          state.edges = state.edges.filter(edge => edge.source !== id && edge.target !== id);
          state.selectedNodes = state.selectedNodes.filter(nodeId => nodeId !== id);
        });
      },

      addEdge: (edge: Edge) => {
        set(state => {
          state.edges.push(edge);
        });
      },

      selectNodes: (nodeIds: string[]) => {
        set(state => {
          state.selectedNodes = nodeIds;
        });
      },

      clearSelection: () => {
        set(state => {
          state.selectedNodes = [];
          state.selectedEdges = [];
        });
      },

      setViewport: (viewport: Viewport) => {
        set(state => {
          state.viewport = viewport;
        });
      },
    })),
    {
      name: 'workflow-store',
    }
  )
);
```

#### AI Agent 状态管理

```typescript
// stores/agentStore.ts
interface AgentState {
  agents: AIAgent[];
  currentAgent: AIAgent | null;

  // 模板和预设
  agentTemplates: AgentTemplate[];
  modelProviders: ModelProvider[];

  // 执行状态
  runningAgents: Record<string, AgentExecution>;
  executionHistory: AgentExecution[];

  // Actions
  createAgent: (template?: AgentTemplate) => void;
  updateAgent: (id: string, updates: Partial<AIAgent>) => void;
  deleteAgent: (id: string) => void;
  duplicateAgent: (id: string) => void;

  // 执行操作
  runAgent: (id: string, input: any) => Promise<void>;
  stopAgent: (id: string) => void;
  getExecutionHistory: (agentId: string) => AgentExecution[];

  // 配置管理
  loadModelProviders: () => Promise<void>;
  saveAgentConfig: (id: string) => Promise<void>;
}

export const useAgentStore = create<AgentState>()(
  devtools(
    immer((set, get) => ({
      agents: [],
      currentAgent: null,
      agentTemplates: [],
      modelProviders: [],
      runningAgents: {},
      executionHistory: [],

      createAgent: (template?: AgentTemplate) => {
        const newAgent: AIAgent = {
          id: generateId(),
          name: template?.name || '新智能体',
          description: template?.description || '',
          type: template?.type || 'chat',
          config: template?.defaultConfig || {},
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        };

        set(state => {
          state.agents.push(newAgent);
          state.currentAgent = newAgent;
        });
      },

      runAgent: async (id: string, input: any) => {
        const agent = get().agents.find(a => a.id === id);
        if (!agent) return;

        const execution: AgentExecution = {
          id: generateId(),
          agentId: id,
          input,
          startTime: new Date().toISOString(),
          status: 'running',
        };

        set(state => {
          state.runningAgents[id] = execution;
        });

        try {
          // TODO: 调用 AI Agent API
          const result = await executeAgent(agent, input);

          set(state => {
            const execution = state.runningAgents[id];
            if (execution) {
              execution.status = 'completed';
              execution.endTime = new Date().toISOString();
              execution.output = result;
            }
            state.executionHistory.push(execution);
            delete state.runningAgents[id];
          });
        } catch (error) {
          set(state => {
            const execution = state.runningAgents[id];
            if (execution) {
              execution.status = 'failed';
              execution.endTime = new Date().toISOString();
              execution.error = error.message;
            }
            state.executionHistory.push(execution);
            delete state.runningAgents[id];
          });
        }
      },
    })),
    {
      name: 'agent-store',
    }
  )
);
```

### 5.2 持久化中间件

```typescript
// utils/storage.ts
import { persist, createJSONStorage } from 'zustand/middleware';

// 自定义存储中间件，支持加密
export const createSecureStorage = (key: string) => {
  return createJSONStorage(() => ({
    getItem: (name: string) => {
      const item = localStorage.getItem(`${key}:${name}`);
      if (item) {
        try {
          // TODO: 解密逻辑
          return JSON.parse(item);
        } catch {
          return null;
        }
      }
      return null;
    },
    setItem: (name: string, value: any) => {
      try {
        // TODO: 加密逻辑
        localStorage.setItem(`${key}:${name}`, JSON.stringify(value));
      } catch (error) {
        console.error('Failed to save to storage:', error);
      }
    },
    removeItem: (name: string) => {
      localStorage.removeItem(`${key}:${name}`);
    },
  }));
};

// 在 Store 中使用持久化
export const usePersistedWorkflowStore = create<WorkflowState>()(
  devtools(
    persist(
      immer((set, get) => ({
        // ... store state 和 actions
      })),
      {
        name: 'hetumind-workflow-storage',
        storage: createSecureStorage('hetumind'),
        partialize: state => ({
          workflows: state.workflows,
          currentWorkflowId: state.currentWorkflowId,
        }),
      }
    ),
    {
      name: 'workflow-store',
    }
  )
);
```

### 5.3 数据与状态边界约定（React Query vs Zustand）

- React Query 管“外部数据”：API 拉取、缓存、同步与重试；组件通过 `useQuery/useMutation` 读写后端。
- Zustand 管“内部状态”：UI 选择、临时配置、编辑器交互过程（如选中节点、视图缩放）。
- 事件驱动交互：
  - 外部数据加载完成后，以事件/副作用方式写入必要的内部状态。
  - 内部状态变更触发业务操作时，通过 mutation 更新后端，成功后同步缓存。
- 保持简单：避免将后端数据整体镜像到 Zustand；仅在需要的地方将片段态写入 Store，以降低耦合与复杂度。

```typescript
// pages/Workflows/Editor.tsx（示例：React Query + 事件驱动写入局部 Zustand）
import { useWorkflow } from '@/services/hooks';

const WorkflowEditorPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading } = useWorkflow(id!);
  const { setNodes, setEdges } = useWorkflowStore();

  useEffect(() => {
    if (data) {
      // 仅将画布渲染所需的片段写入 Zustand；其余原始数据保留在 React Query 缓存中
      setNodes(data.nodes);
      setEdges(data.edges);
    }
  }, [data, setNodes, setEdges]);

  // ... 组件渲染逻辑
};
```

## 6. 页面路由设计

### 6.1 路由结构

```typescript
// router/index.tsx
import React from 'react';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import MainLayout from '@/components/layout/MainLayout';

const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/',
    element: <MainLayout />,
    children: [
      {
        index: true,
        element: <Navigate to="/dashboard" replace />,
      },
      {
        path: 'dashboard',
        element: <DashboardPage />,
      },
      {
        path: 'workflows',
        children: [
          {
            index: true,
            element: <WorkflowListPage />,
          },
          {
            path: ':id',
            element: <WorkflowEditorPage />,
          },
          {
            path: ':id/executions',
            element: <WorkflowExecutionsPage />,
          },
        ],
      },
      {
        path: 'agents',
        children: [
          {
            index: true,
            element: <AgentListPage />,
          },
          {
            path: 'new',
            element: <AgentEditorPage />,
          },
          {
            path: ':id',
            element: <AgentEditorPage />,
          },
          {
            path: ':id/test',
            element: <AgentTestPage />,
          },
        ],
      },
      {
        path: 'settings',
        children: [
          {
            index: true,
            element: <GeneralSettingsPage />,
          },
          {
            path: 'models',
            element: <ModelSettingsPage />,
          },
          {
            path: 'integrations',
            element: <IntegrationSettingsPage />,
          },
          {
            path: 'credentials',
            element: <CredentialsSettingsPage />, // 凭据中心 UI（掩码展示与校验）
          },
        ],
      },
    ],
  },
  {
    path: '*',
    element: <NotFoundPage />,
  },
]);

const AppRouter: React.FC = () => {
  return <RouterProvider router={router} />;
};

export default AppRouter;
```

### 6.2 主要页面组件

#### 仪表板页面

```typescript
// pages/Dashboard/index.tsx
import React from 'react';
import { Row, Col, Card, Statistic, Progress, List, Avatar, Tag } from 'antd';
import { RobotOutlined, PlayCircleOutlined, CheckCircleOutlined, ClockCircleOutlined } from '@ant-design/icons';

const DashboardPage: React.FC = () => {
  // 统一用 react-query 获取外部数据
  const { data: agents = [] } = useQuery({
    queryKey: ['agents'],
    queryFn: () => fetch('/api/v1/agents').then(r => r.json()),
  });
  const { data: workflows = [] } = useQuery({
    queryKey: ['workflows'],
    queryFn: () => fetch('/api/v1/workflows').then(r => r.json()),
  });
  const { data: executionHistory = [] } = useQuery({
    queryKey: ['executions'],
    queryFn: () => fetch('/api/v1/executions').then(r => r.json()),
  });

  const stats = {
    totalAgents: agents.length,
    activeAgents: agents.filter(a => a.status === 'active').length,
    totalWorkflows: workflows.length,
    recentExecutions: executionHistory.filter(e => new Date(e.startTime) > new Date(Date.now() - 24 * 60 * 60 * 1000))
      .length,
  };

  return (
    <div className="dashboard-page">
      <Row gutter={[16, 16]}>
        {/* 统计卡片 */}
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="AI 智能体"
              value={stats.totalAgents}
              prefix={<RobotOutlined />}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="活跃智能体"
              value={stats.activeAgents}
              prefix={<CheckCircleOutlined />}
              valueStyle={{ color: '#52c41a' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="工作流"
              value={stats.totalWorkflows}
              prefix={<PlayCircleOutlined />}
              valueStyle={{ color: '#fa8c16' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="24小时执行"
              value={stats.recentExecutions}
              prefix={<ClockCircleOutlined />}
              valueStyle={{ color: '#722ed1' }}
            />
          </Card>
        </Col>
      </Row>

      <Row gutter={[16, 16]} style={{ marginTop: 16 }}>
        {/* 最近活动 */}
        <Col xs={24} lg={12}>
          <Card title="最近执行" extra={<Button type="link">查看全部</Button>}>
            <List
              dataSource={executionHistory.slice(0, 5)}
              renderItem={execution => (
                <List.Item>
                  <List.Item.Meta
                    avatar={<Avatar icon={<RobotOutlined />} />}
                    title={agents.find(a => a.id === execution.agentId)?.name}
                    description={new Date(execution.startTime).toLocaleString()}
                  />
                  <div>
                    <Tag color={execution.status === 'completed' ? 'green' : 'orange'}>{execution.status}</Tag>
                  </div>
                </List.Item>
              )}
            />
          </Card>
        </Col>

        {/* 热门工作流 */}
        <Col xs={24} lg={12}>
          <Card title="热门工作流" extra={<Button type="link">查看全部</Button>}>
            <List
              dataSource={workflows.slice(0, 5)}
              renderItem={workflow => (
                <List.Item>
                  <List.Item.Meta title={workflow.name} description={workflow.description} />
                  <div>
                    <Progress percent={workflow.successRate || 0} size="small" format={percent => `${percent}%`} />
                  </div>
                </List.Item>
              )}
            />
          </Card>
        </Col>
      </Row>
    </div>
  );
};
```

#### 工作流编辑器页面

```typescript
// pages/Workflows/Editor.tsx
import React, { useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Button, Space, Breadcrumb, message } from 'antd';
import { SaveOutlined, PlayCircleOutlined, ArrowLeftOutlined } from '@ant-design/icons';

const WorkflowEditorPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: workflow, isLoading } = useWorkflow(id!);
  const saveMutation = useSaveWorkflow();
  const runMutation = useRunWorkflow();
  const { nodes, edges } = useWorkflowStore();

  // 通过 React Query 加载外部数据；必要片段写入 Zustand 由上文 5.3 约定处理

  const handleSave = useCallback(async () => {
    try {
      // 函数级注释：保存工作流（使用 mutation，包含乐观更新与统一错误提示）
      await saveMutation.mutateAsync({ id: id!, data: { nodes, edges } });
      message.success('工作流保存成功');
    } catch (error: any) {
      message.error('保存失败: ' + (error?.message || '未知错误'));
    }
  }, [saveMutation, id, nodes, edges]);

  const handleRun = useCallback(async () => {
    try {
      // 函数级注释：运行工作流（统一错误提示）
      await runMutation.mutateAsync({ id: id! });
      message.success('工作流执行开始');
    } catch (error: any) {
      message.error('执行失败: ' + (error?.message || '未知错误'));
    }
  }, [runMutation, id]);

  return (
    <div className="workflow-editor-page">
      {/* 顶部工具栏 */}
      <div className="editor-toolbar">
        <div className="toolbar-left">
          <Space>
            <Button type="text" icon={<ArrowLeftOutlined />} onClick={() => navigate('/workflows')}>
              返回
            </Button>
            <Breadcrumb>
              <Breadcrumb.Item>工作流</Breadcrumb.Item>
              <Breadcrumb.Item>编辑器</Breadcrumb.Item>
              <Breadcrumb.Item>{workflow?.name}</Breadcrumb.Item>
            </Breadcrumb>
          </Space>
        </div>

        <div className="toolbar-right">
          <Space>
            <Button type="primary" icon={<SaveOutlined />} onClick={handleSave}>
              保存
            </Button>
            <Button type="default" icon={<PlayCircleOutlined />} onClick={handleRun}>
              运行
            </Button>
          </Space>
        </div>
      </div>

      {/* 主要内容区域 */}
      <div className="editor-content">
        <div className="editor-layout">
          {/* 左侧节点面板 */}
          <div className="editor-sidebar">
            <NodeLibrary />
          </div>

          {/* 中间画布区域 */}
          <div className="editor-canvas">
            <WorkflowCanvas workflowId={id!} />
          </div>

          {/* 右侧属性面板 */}
          <div className="editor-properties">
            <PropertyPanel />
          </div>
        </div>
      </div>
    </div>
  );
};
```

#### 智能体列表页面（react-query + mutation）

```typescript
// pages/Agents/index.tsx
import React from 'react';
import { Table, Button, Space, message } from 'antd';
import { useAgents, useCreateAgent, useDeleteAgent } from '@/services/hooks';

const AgentListPage: React.FC = () => {
  const { data: agents = [], isLoading } = useAgents();
  const createMutation = useCreateAgent();
  const deleteMutation = useDeleteAgent();

  /* 函数级注释：创建 Agent（成功后自动刷新列表） */
  const handleCreate = async () => {
    try {
      await createMutation.mutateAsync({ name: '新智能体', type: 'chat' });
      message.success('创建成功');
    } catch (e: any) {
      message.error('创建失败：' + (e?.message || '未知错误'));
    }
  };

  /* 函数级注释：删除 Agent（包含乐观更新，失败自动回滚） */
  const handleDelete = async (id: string) => {
    try {
      await deleteMutation.mutateAsync({ id });
      message.success('删除成功');
    } catch (e: any) {
      message.error('删除失败：' + (e?.message || '未知错误'));
    }
  };

  return (
    <div>
      <Space style={{ marginBottom: 8 }}>
        <Button type="primary" onClick={handleCreate} loading={createMutation.isPending}>
          新建智能体
        </Button>
      </Space>
      <Table
        rowKey="id"
        loading={isLoading}
        dataSource={agents}
        columns={[
          { title: '名称', dataIndex: 'name' },
          { title: '类型', dataIndex: 'type' },
          {
            title: '操作',
            render: (_: any, rec: any) => (
              <Space>
                <Button danger size="small" onClick={() => handleDelete(rec.id)} loading={deleteMutation.isPending}>
                  删除
                </Button>
              </Space>
            ),
          },
        ]}
      />
    </div>
  );
};
```

#### 工作流列表页面（react-query + 详情加载）

```typescript
// pages/Workflows/index.tsx
import React from 'react';
import { List } from 'antd';
import { Link } from 'react-router-dom';
import { useWorkflows } from '@/services/hooks';

const WorkflowListPage: React.FC = () => {
  const { data: workflows = [], isLoading } = useWorkflows();
  return (
    <List
      loading={isLoading}
      dataSource={workflows}
      renderItem={(wf: any) => (
        <List.Item actions={[<Link to={`/workflows/${wf.id}`}>编辑</Link>]}>
          <List.Item.Meta title={wf.name} description={wf.description} />
        </List.Item>
      )}
    />
  );
};
```

#### 执行历史页面（react-query + 写操作 mutation）

```typescript
// pages/Workflows/Executions.tsx
import React from 'react';
import { Table, Button, Space, Tag, message } from 'antd';
import { useExecutions, useCancelExecution, useRetryExecution } from '@/services/hooks';

const WorkflowExecutionsPage: React.FC<{ workflowId: string }> = ({ workflowId }) => {
  const { data: rows = [], isLoading } = useExecutions(workflowId);
  const cancelMutation = useCancelExecution();
  const retryMutation = useRetryExecution();

  /* 函数级注释：取消执行（统一错误提示） */
  const cancelExec = async (id: string) => {
    try {
      await cancelMutation.mutateAsync({ id });
      message.success('已取消');
    } catch (e: any) {
      message.error('取消失败：' + (e?.message || '未知错误'));
    }
  };

  /* 函数级注释：重试执行（统一错误提示） */
  const retryExec = async (id: string) => {
    try {
      await retryMutation.mutateAsync({ id });
      message.success('已重试');
    } catch (e: any) {
      message.error('重试失败：' + (e?.message || '未知错误'));
    }
  };

  return (
    <Table
      rowKey="id"
      loading={isLoading}
      dataSource={rows}
      columns={[
        { title: '开始时间', dataIndex: 'startTime' },
        { title: '结束时间', dataIndex: 'endTime' },
        { title: '耗时(ms)', dataIndex: 'durationMs' },
        {
          title: '状态',
          dataIndex: 'status',
          render: (s: string) => <Tag color={s === 'completed' ? 'green' : 'orange'}>{s}</Tag>,
        },
        {
          title: '操作',
          render: (_: any, rec: any) => (
            <Space>
              <Button size="small" onClick={() => retryExec(rec.id)} loading={retryMutation.isPending}>
                重试
              </Button>
              <Button danger size="small" onClick={() => cancelExec(rec.id)} loading={cancelMutation.isPending}>
                取消
              </Button>
            </Space>
          ),
        },
      ]}
    />
  );
};
```

## 7. 核心功能实现

### 7.1 数据流映射系统

#### 拖拽数据映射（统一使用 dnd-kit）

```typescript
// components/workflow/DataMapping.tsx（dnd-kit 版本）
import React, { useState } from 'react';
import { DndContext, useDraggable, useDroppable } from '@dnd-kit/core';
import { Button, Select, Space, Tag } from 'antd';

interface DataMappingProps {
  sourceNode: Node;
  targetNode: Node;
  onMappingChange: (mappings: DataMapping[]) => void;
}

/* 函数级注释：字段项可拖拽渲染器 */
const DraggableField: React.FC<{ id: string; label: string }> = ({ id, label }) => {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id });
  return (
    <div ref={setNodeRef} {...attributes} {...listeners} className={`field-item ${isDragging ? 'dragging' : ''}`}>
      <Space>
        <span>{label}</span>
      </Space>
    </div>
  );
};

/* 函数级注释：目标区域用于接收映射 */
const TargetDropZone: React.FC<{ onDrop: (sourceField: string) => void }> = ({ onDrop }) => {
  const { setNodeRef, isOver } = useDroppable({ id: 'target-zone' });
  return (
    <div ref={setNodeRef} className={`target-zone ${isOver ? 'drop-active' : ''}`}>
      放到此处创建映射
    </div>
  );
};

const DataMapping: React.FC<DataMappingProps> = ({ sourceNode, targetNode, onMappingChange }) => {
  const [mappings, setMappings] = useState<DataMapping[]>([]);

  const handleDragEnd = (event: any) => {
    const sourceField = event.active?.id as string;
    const overId = event.over?.id as string;
    if (sourceField && overId === 'target-zone') {
      const newMapping: DataMapping = {
        id: generateId(),
        sourceField,
        targetField: 'target-zone',
        transform: 'direct',
      };
      const next = [...mappings, newMapping];
      setMappings(next);
      onMappingChange(next);
    }
  };

  const updateMapping = (id: string, patch: Partial<DataMapping>) => {
    const next = mappings.map(m => (m.id === id ? { ...m, ...patch } : m));
    setMappings(next);
    onMappingChange(next);
  };

  const removeMapping = (id: string) => {
    const next = mappings.filter(m => m.id !== id);
    setMappings(next);
    onMappingChange(next);
  };

  return (
    <DndContext onDragEnd={handleDragEnd}>
      <div className="data-mapping">
        <div className="source-fields">
          <h4>源数据字段</h4>
          {sourceNode.outputFields?.map(f => (
            <DraggableField key={f.name} id={f.name} label={`${f.name} (${f.type})`} />
          ))}
        </div>

        <div className="mapping-area">
          <h4>数据映射</h4>
          {mappings.map(m => (
            <div key={m.id} className="mapping-item">
              <Space>
                <span>{m.sourceField}</span>
                <span>→</span>
                <span>{m.targetField}</span>
                <Select size="small" value={m.transform} onChange={val => updateMapping(m.id, { transform: val })}>
                  <Select.Option value="direct">直接映射</Select.Option>
                  <Select.Option value="function">函数转换</Select.Option>
                  <Select.Option value="expression">表达式</Select.Option>
                </Select>
                <Button type="text" size="small" onClick={() => removeMapping(m.id)}>
                  删除
                </Button>
              </Space>
            </div>
          ))}
        </div>

        <div className="target-fields">
          <h4>目标字段</h4>
          <TargetDropZone
            onDrop={() => {
              /* 留空，统一在 handleDragEnd 中处理 */
            }}
          />
          {targetNode.inputFields?.map(f => (
            <div key={f.name} className="field-item target">
              <Space>
                <div>
                  <div className="field-name">{f.name}</div>
                  <div className="field-type">{f.type}</div>
                </div>
                {f.required && <Tag color="red">必填</Tag>}
              </Space>
            </div>
          ))}
        </div>
      </div>
    </DndContext>
  );
};
export default DataMapping;
```

### 7.2 表达式编辑器

```typescript
// components/common/ExpressionEditor.tsx
import React, { useRef, useEffect, useState } from 'react';
import Editor from '@monaco-editor/react';

interface ExpressionEditorProps {
  value: string;
  onChange: (value: string) => void;
  onExecute?: (expression: string) => any;
  variables?: Variable[];
  functions?: Function[];
}

const ExpressionEditor: React.FC<ExpressionEditorProps> = ({
  value,
  onChange,
  onExecute,
  variables = [],
  functions = [],
}) => {
  const editorRef = useRef<any>(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const [result, setResult] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  // 自定义语言定义
  const expressionLanguage = {
    // 定义表达式语法高亮
    tokenizer: {
      root: [
        // 变量引用 {{variable}}
        [/\{\{.*?\}\}/, 'variable'],

        // 函数调用 function()
        [/\b\w+(?=\s*\()/, 'function'],

        // 数字
        [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
        [/\d+/, 'number'],

        // 字符串
        [/"([^"\\]|\\.)*$/, 'string.invalid'],
        [/"/, 'string', '@string_double'],
        [/'([^'\\]|\\.)*$/, 'string.invalid'],
        [/'/, 'string', '@string_single'],

        // 运算符
        [/[<>]=?/, 'operator'],
        [/[+\-*/%]/, 'operator'],
        [/[!=]=/, 'operator'],

        // 关键字
        [/\b(true|false|null|undefined)\b/, 'keyword'],

        // 标识符
        [/\w+/, 'identifier'],
      ],

      string_double: [
        [/[^\\"]+/, 'string'],
        [/\\./, 'string.escape'],
        [/"/, 'string', '@pop'],
      ],

      string_single: [
        [/[^\\']+/, 'string'],
        [/\\./, 'string.escape'],
        [/'/, 'string', '@pop'],
      ],
    },
  };

  // 自动补全
  const provideCompletionItems = (model: any, position: any) => {
    const suggestions = [];

    // 变量补全
    variables.forEach(variable => {
      suggestions.push({
        label: variable.name,
        kind: monaco.languages.CompletionItemKind.Variable,
        insertText: `{{${variable.name}}}`,
        documentation: variable.description,
      });
    });

    // 函数补全
    functions.forEach(func => {
      suggestions.push({
        label: func.name,
        kind: monaco.languages.CompletionItemKind.Function,
        insertText: `${func.name}()`,
        documentation: func.description,
      });
    });

    return { suggestions };
  };

  const handleExecute = async () => {
    if (!onExecute) return;

    setIsExecuting(true);
    setError(null);

    try {
      const result = await onExecute(value);
      setResult(result);
    } catch (err) {
      setError(err.message);
    } finally {
      setIsExecuting(false);
    }
  };

  return (
    <div className="expression-editor">
      <div className="editor-toolbar">
        <Space>
          <Button size="small" type="primary" loading={isExecuting} onClick={handleExecute}>
            执行表达式
          </Button>

          {/* 变量快速插入 */}
          <Dropdown
            menu={{
              items: variables.map(variable => ({
                key: variable.name,
                label: variable.name,
                onClick: () => onChange(value + `{{${variable.name}}}`),
              })),
            }}
            trigger={['click']}
          >
            <Button size="small">插入变量</Button>
          </Dropdown>
        </Space>
      </div>

      <div className="editor-container">
        <Editor
          height="200px"
          defaultLanguage="expression"
          value={value}
          onChange={newValue => onChange(newValue || '')}
          onMount={(editor, monaco) => {
            editorRef.current = editor;

            // 注册自定义语言（使用 onMount 提供的 monaco 实例）
            monaco.languages.register({ id: 'expression' });
            monaco.languages.setMonarchTokensProvider('expression', expressionLanguage);

            // 设置自动补全
            monaco.languages.registerCompletionItemProvider('expression', {
              provideCompletionItems,
            });
          }}
          options={{
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            fontSize: 14,
            lineNumbers: 'on',
            renderWhitespace: 'selection',
            automaticLayout: true,
          }}
        />
      </div>

      {/* 执行结果 */}
      {(result !== null || error) && (
        <div className="execution-result">
          {error ? (
            <div className="error">
              <Text type="danger">错误: {error}</Text>
            </div>
          ) : (
            <div className="result">
              <Text>结果: {JSON.stringify(result)}</Text>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
```

## 8. 性能优化策略

### 8.1 画布性能优化

```typescript
// hooks/useCanvasPerformance.ts
export const useCanvasPerformance = () => {
  const [isOptimized, setIsOptimized] = useState(false);

  // 节点虚拟化
  const useNodeVirtualization = (nodes: Node[], viewport: Viewport) => {
    return useMemo(() => {
      if (!isOptimized || nodes.length < 100) return nodes;

      const visibleBounds = {
        left: -viewport.x / viewport.zoom,
        top: -viewport.y / viewport.zoom,
        right: (-viewport.x + window.innerWidth) / viewport.zoom,
        bottom: (-viewport.y + window.innerHeight) / viewport.zoom,
      };

      return nodes.filter(
        node =>
          node.position.x < visibleBounds.right &&
          node.position.x + 200 > visibleBounds.left &&
          node.position.y < visibleBounds.bottom &&
          node.position.y + 100 > visibleBounds.top
      );
    }, [nodes, viewport, isOptimized]);
  };

  // 连接简化
  const simplifyEdges = (edges: Edge[], viewport: Viewport) => {
    return useMemo(() => {
      if (!isOptimized) return edges;

      // 在低缩放级别时简化连接线渲染
      if (viewport.zoom < 0.5) {
        return edges.map(edge => ({
          ...edge,
          type: 'straight', // 使用简单的直线连接
          animated: false,
        }));
      }

      return edges;
    }, [edges, viewport, isOptimized]);
  };

  // 防抖更新
  const debouncedUpdate = useDebouncedCallback(
    (updateFn: () => void) => updateFn(),
    16 // 60fps
  );

  return {
    isOptimized,
    setIsOptimized,
    useNodeVirtualization,
    simplifyEdges,
    debouncedUpdate,
  };
};
```

### 8.2 内存管理

```typescript
// utils/memory.ts
export class MemoryManager {
  private static cache = new Map<string, any>();
  private static maxCacheSize = 100;

  // LRU 缓存
  static set(key: string, value: any) {
    if (this.cache.size >= this.maxCacheSize) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
    this.cache.set(key, value);
  }

  static get(key: string): any {
    const value = this.cache.get(key);
    if (value !== undefined) {
      // 重新插入以更新 LRU 顺序
      this.cache.delete(key);
      this.cache.set(key, value);
    }
    return value;
  }

  // 清理缓存
  static clear() {
    this.cache.clear();
  }

  // 监控内存使用
  static getMemoryUsage() {
    if (performance.memory) {
      return {
        used: performance.memory.usedJSHeapSize,
        total: performance.memory.totalJSHeapSize,
        limit: performance.memory.jsHeapSizeLimit,
      };
    }
    return null;
  }
}

// 在组件中使用内存管理
const useMemoryOptimization = () => {
  useEffect(() => {
    // 定期清理缓存
    const interval = setInterval(() => {
      MemoryManager.clear();
    }, 5 * 60 * 1000); // 每5分钟清理一次

    return () => clearInterval(interval);
  }, []);

  const memoizedValue = useMemo(() => {
    // 使用缓存
    return MemoryManager.get('expensive-computation') || computeExpensiveValue();
  }, []);

  return { memoizedValue };
};
```

## 9. 安全性考虑

### 9.1 XSS 防护

```typescript
// utils/security.ts
export const sanitizeExpression = (expression: string): string => {
  // 清理用户输入的表达式，防止 XSS
  return expression
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
    .replace(/javascript:/gi, '')
    .replace(/on\w+\s*=/gi, '');
};

export const validateNodeConfig = (config: Record<string, any>): boolean => {
  // 验证节点配置的安全性
  const dangerousPatterns = [/<script/i, /javascript:/i, /on\w+\s*=/i, /eval\s*\(/i, /Function\s*\(/i];

  const configString = JSON.stringify(config);
  return !dangerousPatterns.some(pattern => pattern.test(configString));
};
```

### 9.2 API 安全

```typescript
// services/api.ts
class APIClient {
  private baseURL: string;
  private token: string | null = null;

  constructor(baseURL: string) {
    this.baseURL = baseURL;
  }

  setToken(token: string) {
    this.token = token;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseURL}${endpoint}`;
    const headers = {
      'Content-Type': 'application/json',
      ...(this.token && { Authorization: `Bearer ${this.token}` }),
      ...options.headers,
    };

    try {
      const response = await fetch(url, {
        ...options,
        headers,
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('API request failed:', error);
      throw error;
    }
  }

  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint);
  }

  async post<T>(endpoint: string, data: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }
}

export const apiClient = new APIClient('/api');
```

```typescript
// services/api.ts（补充：react-query 集成与统一错误处理）
/* 函数级注释：初始化 QueryClient 并配置统一错误策略 */
import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      refetchOnWindowFocus: false,
    },
  },
});

/* 函数级注释：API 错误统一处理 */
const handleError = (status: number, body: any) => {
  if (status === 401) {
    // 触发登出或刷新逻辑
    // window.location.href = '/login';
  }
};

/* 函数级注释：POST 请求示例，加入错误拦截 */
export async function postJSON<T>(endpoint: string, data: any): Promise<T> {
  const resp = await fetch(`/api${endpoint}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!resp.ok) {
    const body = await resp.text();
    handleError(resp.status, body);
    throw new Error(`HTTP ${resp.status}: ${resp.statusText}`);
  }
  return resp.json();
}
```

```typescript
// services/hooks.ts（示例：用 react-query 统一数据获取层）
import { useQuery } from '@tanstack/react-query';

/* 函数级注释：加载工作流详情 */
export function useWorkflow(id: string) {
  return useQuery({
    queryKey: ['workflow', id],
    queryFn: async () => {
      const resp = await fetch(`/api/v1/workflows/${id}`);
      if (!resp.ok) throw new Error('Failed to load workflow');
      return resp.json();
    },
    retry: 2,
    refetchOnWindowFocus: false,
  });
}
```

```typescript
// services/hooks.ts（扩展：列表/详情/写操作）
import { useQuery, useMutation } from '@tanstack/react-query';
import { queryClient } from '@/services/api';

/* 函数级注释：加载 Agents 列表 */
export function useAgents() {
  return useQuery({ queryKey: ['agents'], queryFn: async () => (await fetch('/api/v1/agents')).json() });
}

/* 函数级注释：加载 Workflows 列表 */
export function useWorkflows() {
  return useQuery({ queryKey: ['workflows'], queryFn: async () => (await fetch('/api/v1/workflows')).json() });
}

/* 函数级注释：加载执行历史（按工作流） */
export function useExecutions(workflowId: string) {
  return useQuery({
    queryKey: ['executions', workflowId],
    queryFn: async () => (await fetch(`/api/v1/workflows/${workflowId}/executions`)).json(),
  });
}

/* 函数级注释：创建 Agent（成功后刷新列表） */
export function useCreateAgent() {
  return useMutation({
    mutationFn: async (payload: any) => {
      const resp = await fetch('/api/v1/agents', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      if (!resp.ok) throw new Error('Create agent failed');
      return resp.json();
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['agents'] }),
  });
}

/* 函数级注释：删除 Agent（乐观更新 + 回滚） */
export function useDeleteAgent() {
  return useMutation({
    mutationFn: async ({ id }: { id: string }) => {
      const resp = await fetch(`/api/v1/agents/${id}`, { method: 'DELETE' });
      if (!resp.ok) throw new Error('Delete agent failed');
      return { id };
    },
    onMutate: async ({ id }) => {
      await queryClient.cancelQueries({ queryKey: ['agents'] });
      const prev = queryClient.getQueryData<any[]>(['agents']);
      queryClient.setQueryData<any[]>(['agents'], (old = []) => old.filter(a => a.id !== id));
      return { prev };
    },
    onError: (_err, _vars, ctx) => {
      if (ctx?.prev) queryClient.setQueryData(['agents'], ctx.prev);
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: ['agents'] }),
  });
}

/* 函数级注释：保存工作流（乐观更新 + 回滚） */
export function useSaveWorkflow() {
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: any }) => {
      const resp = await fetch(`/api/v1/workflows/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      });
      if (!resp.ok) throw new Error('Save workflow failed');
      return resp.json();
    },
    onMutate: async ({ id, data }) => {
      await queryClient.cancelQueries({ queryKey: ['workflow', id] });
      const prev = queryClient.getQueryData(['workflow', id]);
      queryClient.setQueryData(['workflow', id], (old: any) => ({ ...(old || {}), ...data }));
      return { prev };
    },
    onError: (_err, { id }, ctx) => {
      if (ctx?.prev) queryClient.setQueryData(['workflow', id], ctx.prev);
    },
    onSettled: (_data, _err, vars) => queryClient.invalidateQueries({ queryKey: ['workflow', (vars as any).id] }),
  });
}

/* 函数级注释：运行工作流 */
export function useRunWorkflow() {
  return useMutation({
    mutationFn: async ({ id }: { id: string }) => {
      const resp = await fetch(`/api/v1/workflows/${id}/run`, { method: 'POST' });
      if (!resp.ok) throw new Error('Run workflow failed');
      return resp.json();
    },
  });
}

/* 函数级注释：取消执行 */
export function useCancelExecution() {
  return useMutation({
    mutationFn: async ({ id }: { id: string }) => {
      const resp = await fetch(`/api/v1/executions/${id}/cancel`, { method: 'POST' });
      if (!resp.ok) throw new Error('Cancel execution failed');
      return resp.json();
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['executions'] }),
  });
}

/* 函数级注释：重试执行 */
export function useRetryExecution() {
  return useMutation({
    mutationFn: async ({ id }: { id: string }) => {
      const resp = await fetch(`/api/v1/executions/${id}/retry`, { method: 'POST' });
      if (!resp.ok) throw new Error('Retry execution failed');
      return resp.json();
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['executions'] }),
  });
}
```

## 10. 测试策略

### 10.1 单元测试

```typescript
// __tests__/components/WorkflowCanvas.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { WorkflowCanvas } from '@/components/workflow/WorkflowCanvas';

describe('WorkflowCanvas', () => {
  it('should render canvas with controls', () => {
    render(<WorkflowCanvas workflowId="test-id" />);

    expect(screen.getByTestId('workflow-canvas')).toBeInTheDocument();
    expect(screen.getByTestId('canvas-controls')).toBeInTheDocument();
    expect(screen.getByTestId('canvas-minimap')).toBeInTheDocument();
  });

  it('should add node when dragging from panel', () => {
    const { container } = render(<WorkflowCanvas workflowId="test-id" />);

    const canvas = screen.getByTestId('workflow-canvas');
    const nodeLibrary = screen.getByTestId('node-library');

    // 模拟拖拽操作
    fireEvent.dragStart(nodeLibrary.querySelector('[data-node-type="aiAgent"]')!);
    fireEvent.dragOver(canvas);
    fireEvent.drop(canvas);

    expect(screen.getByText('AI 智能体')).toBeInTheDocument();
  });

  it('should create connection between nodes', () => {
    render(<WorkflowCanvas workflowId="test-id" />);

    // 添加两个节点
    // 模拟连接操作
    // 验证连接是否创建成功
  });
});
```

### 10.2 集成测试

```typescript
// __tests__/integration/workflow-execution.test.ts
describe('Workflow Execution Integration', () => {
  it('should execute complete workflow with AI agent', async () => {
    // 创建工作流
    const workflow = createTestWorkflow();

    // 配置 AI Agent
    const agent = configureTestAgent();

    // 执行工作流
    const result = await executeWorkflow(workflow, { input: 'test input' });

    // 验证结果
    expect(result.status).toBe('completed');
    expect(result.output).toBeDefined();
  });
});
```

## 11. 部署配置

### 11.1 构建配置

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: process.env.NODE_ENV === 'development',
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          antd: ['antd', '@ant-design/icons'],
          monaco: ['@monaco-editor/react'],
          dnd: ['@dnd-kit/core', '@dnd-kit/sortable'],
          xyflow: ['@xyflow/react'],
        },
      },
    },
  },
  server: {
    port: 3001, // 避免与 hetuflow-web 冲突
    proxy: {
      '/api': {
        target: 'http://localhost:9501', // hetumind-studio 端口
        changeOrigin: true,
        secure: false,
      },
    },
  },
  optimizeDeps: {
    include: ['react', 'react-dom', 'antd'],
  },
});
```

// 注：前端开发与部署不使用 Docker，直接使用 Node（>=22）与 pnpm 进行开发与构建。

## 12. 开发计划

### 12.1 开发阶段

**第一阶段：基础框架搭建**

- [x] 项目初始化和基础配置
- [x] 主题系统实现
- [x] 布局组件开发
- [x] 路由系统配置
- [x] 状态管理设置

**第二阶段：核心功能开发**

- [ ] 工作流画布基础功能
- [ ] 节点拖拽和连接
- [ ] 属性面板开发
- [ ] AI Agent 配置界面

**第三阶段：高级功能**

- [ ] 数据映射系统
- [ ] 表达式编辑器
- [ ] 工作流执行引擎
- [ ] 实时监控面板

**第四阶段：优化和完善**

- [ ] 性能优化
- [ ] 测试覆盖
- [ ] 文档完善
- [ ] 部署配置

### 12.2 技术里程碑

1. **Week 1-2**: 基础框架完成，支持主题切换
2. **Week 3-4**: 工作流画布基本功能
3. **Week 5-6**: 节点系统和属性编辑
4. **Week 7-8**: AI Agent 配置和测试
5. **Week 9-10**: 数据映射和表达式系统
6. **Week 11-12**: 性能优化和测试

## 13. 总结

本技术方案为 Hetumind Web 前端项目提供了完整的架构设计，基于现代化的 React 19 + TypeScript + Ant Design 5 技术栈，实现了：

### 13.1 核心特性

- **可视化工作流编辑器**: 基于 React Flow 的强大画布系统
- **AI Agent 开发平台**: 直观的智能体配置和管理界面
- **现代化主题系统**: 支持亮色/暗色主题和多色彩方案
- **模块化架构**: 清晰的组件分层和状态管理
- **高性能优化**: 虚拟化、缓存和内存管理策略

### 13.2 技术优势

- **类型安全**: 完整的 TypeScript 类型覆盖
- **开发体验**: 热重载、ESLint、Prettier 等工具链
- **可扩展性**: 插件化架构支持功能扩展
- **性能优化**: 多层次的渲染和内存优化
- **安全性**: XSS 防护和输入验证机制

### 13.3 下一步行动

1. 根据此方案创建项目基础结构
2. 实现核心组件和页面
3. 集成 hetumind-studio 后端 API
4. 完善功能特性并进行测试

该技术方案为 Hetumind Web 项目奠定了坚实的技术基础，将为用户提供强大、直观、高效的 AI Agent 开发和工作流编排体验。
