/**
 * 状态管理相关的类型定义
 */

// 基础状态接口
export interface BaseState {
  loading: boolean;
  error: string | null;
  lastUpdated: number | null;
}

// 分页状态
export interface PaginationState {
  current: number;
  pageSize: number;
  total: number;
}

// 排序状态
export interface SortState {
  field?: string;
  order?: 'asc' | 'desc';
}

// 过滤状态
export interface FilterState {
  [key: string]: any;
}

// 搜索状态
export interface SearchState {
  keyword: string;
  filters?: FilterState;
}

// 用户状态
export interface UserState extends BaseState {
  user: {
    id: string;
    username: string;
    email: string;
    avatar?: string;
    roles: string[];
    permissions: string[];
    preferences: UserPreferences;
  } | null;
  isAuthenticated: boolean;
}

// 用户偏好设置
export interface UserPreferences {
  theme: {
    mode: 'light' | 'dark' | 'system';
    colorScheme: 'blue' | 'purple' | 'green' | 'orange';
  };
  language: string;
  autoSave: boolean;
  sidebarCollapsed: boolean;
  editorSettings: {
    fontSize: number;
    tabSize: number;
    wordWrap: boolean;
    minimap: boolean;
    theme: string;
  };
}

// 工作流状态
export interface WorkflowState extends BaseState {
  workflows: Workflow[];
  currentWorkflow: Workflow | null;
  selectedNodes: string[];
  selectedEdges: string[];
  clipboard: {
    nodes: any[];
    edges: any[];
  };
  history: {
    past: Workflow[];
    present: Workflow | null;
    future: Workflow[];
    maxStates: number;
  };
  viewState: {
    zoom: number;
    pan: { x: number; y: number };
    fitView: boolean;
  };
}

// 工作流基础类型
export interface Workflow {
  id: string;
  name: string;
  description?: string;
  version: string;
  status: 'draft' | 'published' | 'archived';
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  variables: WorkflowVariable[];
  settings: WorkflowSettings;
  metadata: WorkflowMetadata;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
  updatedBy: string;
}

// 工作流节点
export interface WorkflowNode {
  id: string;
  type: string;
  data: Record<string, any>;
  position: { x: number; y: number };
  style?: Record<string, any>;
  className?: string;
  sourcePosition?: 'top' | 'right' | 'bottom' | 'left';
  targetPosition?: 'top' | 'right' | 'bottom' | 'left';
}

// 工作流连接
export interface WorkflowEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
  type?: string;
  style?: Record<string, any>;
  label?: string;
  animated?: boolean;
  data?: Record<string, any>;
}

// 工作流变量
export interface WorkflowVariable {
  id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  value: any;
  description?: string;
  required: boolean;
  defaultValue?: any;
}

// 工作流设置
export interface WorkflowSettings {
  timeout?: number;
  retries?: number;
  retryDelay?: number;
  parallel?: boolean;
  maxConcurrency?: number;
  errorHandling?: 'stop' | 'continue' | 'retry';
  notifications?: {
    onSuccess?: boolean;
    onFailure?: boolean;
    onTimeout?: boolean;
    recipients?: string[];
  };
}

// 工作流元数据
export interface WorkflowMetadata {
  tags: string[];
  category?: string;
  version?: string;
  changelog?: string;
  documentation?: string;
}

// Agent 状态
export interface AgentState extends BaseState {
  agents: Agent[];
  currentAgent: Agent | null;
  categories: AgentCategory[];
  templates: AgentTemplate[];
  testResults: AgentTestResult[];
}

// Agent 基础类型
export interface Agent {
  id: string;
  name: string;
  description?: string;
  avatar?: string;
  category: string;
  type: 'chat' | 'workflow' | 'task' | 'custom';
  model: AgentModel;
  config: AgentConfig;
  tools: AgentTool[];
  knowledge: AgentKnowledge[];
  status: 'active' | 'inactive' | 'training' | 'error';
  metrics: AgentMetrics;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
  updatedBy: string;
}

// Agent 模型配置
export interface AgentModel {
  provider: string;
  model: string;
  temperature?: number;
  maxTokens?: number;
  topP?: number;
  frequencyPenalty?: number;
  presencePenalty?: number;
}

// Agent 配置
export interface AgentConfig {
  systemPrompt?: string;
  welcomeMessage?: string;
  instructions?: string[];
  constraints?: string[];
  examples?: AgentExample[];
  memory?: {
    enabled: boolean;
    maxSize?: number;
    retention?: number;
  };
}

// Agent 示例对话
export interface AgentExample {
  input: string;
  output: string;
  context?: string;
}

// Agent 工具
export interface AgentTool {
  id: string;
  name: string;
  description: string;
  type: 'function' | 'api' | 'workflow' | 'search';
  config: Record<string, any>;
  enabled: boolean;
}

// Agent 知识库
export interface AgentKnowledge {
  id: string;
  type: 'document' | 'url' | 'database' | 'api';
  source: string;
  content?: string;
  metadata?: Record<string, any>;
  enabled: boolean;
}

// Agent 分类
export interface AgentCategory {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  color?: string;
  parentId?: string;
  order: number;
}

// Agent 模板
export interface AgentTemplate {
  id: string;
  name: string;
  description?: string;
  category: string;
  type: string;
  config: Partial<Agent>;
  preview?: string;
  tags: string[];
}

// Agent 测试结果
export interface AgentTestResult {
  id: string;
  agentId: string;
  input: string;
  output: string;
  expected?: string;
  status: 'success' | 'error' | 'pending';
  metrics: {
    responseTime: number;
    tokenUsage: {
      input: number;
      output: number;
      total: number;
    };
    cost?: number;
  };
  error?: string;
  createdAt: string;
}

// Agent 指标
export interface AgentMetrics {
  totalConversations: number;
  totalMessages: number;
  averageResponseTime: number;
  successRate: number;
  userSatisfaction?: number;
  tokenUsage: {
    total: number;
    input: number;
    output: number;
  };
  cost?: number;
  lastActiveAt?: string;
}

// 应用全局状态
export interface AppState {
  loading: boolean;
  error: string | null;
  notification: {
    type: 'success' | 'error' | 'warning' | 'info';
    title: string;
    message?: string;
    duration?: number;
    timestamp: number;
  } | null;
  version: string;
  buildTime: string;
  environment: 'development' | 'production' | 'test';
  features: {
    [key: string]: boolean;
  };
}

// UI 状态
export interface UIState {
  sidebarCollapsed: boolean;
  theme: 'light' | 'dark' | 'system';
  colorScheme: 'blue' | 'purple' | 'green' | 'orange';
  language: string;
  modals: {
    [key: string]: boolean;
  };
  drawers: {
    [key: string]: boolean;
  };
  panels: {
    [key: string]: {
      visible: boolean;
      size?: number;
      position?: 'left' | 'right' | 'top' | 'bottom';
    };
  };
  loading: {
    [key: string]: boolean;
  };
}

// 编辑器状态
export interface EditorState {
  activeTab: string;
  tabs: EditorTab[];
  clipboard: any;
  history: {
    past: any[];
    present: any;
    future: any[];
    maxStates: number;
  };
  viewState: {
    zoom: number;
    pan: { x: number; y: number };
    fitView: boolean;
    showGrid: boolean;
    snapToGrid: boolean;
  };
  tools: {
    selected: string;
    config: Record<string, any>;
  };
}

// 编辑器标签页
export interface EditorTab {
  id: string;
  title: string;
  type: 'workflow' | 'agent' | 'code' | 'json' | 'markdown';
  content: any;
  dirty: boolean;
  closable: boolean;
  data?: Record<string, any>;
}

// 执行状态
export interface ExecutionState extends BaseState {
  executions: Execution[];
  currentExecution: Execution | null;
  logs: ExecutionLog[];
  metrics: ExecutionMetrics;
  filters: FilterState;
  pagination: PaginationState;
}

// 执行记录
export interface Execution {
  id: string;
  workflowId: string;
  workflowName: string;
  agentId?: string;
  agentName?: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  input: any;
  output?: any;
  error?: string;
  startTime: string;
  endTime?: string;
  duration?: number;
  metrics: ExecutionMetrics;
  logs: ExecutionLog[];
  triggeredBy: 'manual' | 'schedule' | 'webhook' | 'api';
  triggeredByUser?: string;
  metadata?: Record<string, any>;
}

// 执行日志
export interface ExecutionLog {
  id: string;
  executionId: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  data?: any;
  timestamp: string;
  source?: string;
  nodeId?: string;
  agentId?: string;
}

// 执行指标
export interface ExecutionMetrics {
  duration: number;
  nodeExecutions: number;
  successCount: number;
  errorCount: number;
  retryCount: number;
  tokenUsage?: {
    input: number;
    output: number;
    total: number;
  };
  cost?: number;
  memoryUsage?: {
    peak: number;
    average: number;
  };
  cpuUsage?: {
    peak: number;
    average: number;
  };
}