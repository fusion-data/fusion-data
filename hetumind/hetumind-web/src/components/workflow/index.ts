/**
 * 工作流组件导出
 */

// 主要组件
export { default as WorkflowEditor } from './WorkflowEditor';
export { default as WorkflowCanvas } from './WorkflowCanvas';

// 面板组件
export { default as NodePanel } from './panel/NodePanel';
export { default as PropertyPanel } from './panel/PropertyPanel';

// 工具栏
export { default as WorkflowToolbar } from './toolbar/WorkflowToolbar';

// 节点组件
export {
  BaseNode,
  TriggerNode,
  ActionNode,
  ConditionNode,
  DataProcessorNode,
  WebhookNode,
  TimerNode,
  AIAgentNode,
} from './nodes';

// 节点相关
export {
  NodeProvider,
  useNodeContext,
} from './nodes/NodeContext';

export {
  NodeFactory,
} from './nodes/NodeFactory';

export {
  NodeRegistry,
} from './nodes/NodeRegistry';

export {
  NodeUtils,
} from './nodes/NodeUtils';

// 拖拽组件
export {
  DragDropProvider,
  CanvasDropZone,
  useDragDrop,
} from './dnd/DragDropProvider';

// 类型定义
export type {
  BaseNodeProps,
  NodeConfig,
  NodeTypeConfig,
  NodeTemplate,
  CustomNodeProps,
} from './nodes/types';

// 节点Props类型
export type {
  TriggerNodeProps,
  ActionNodeProps,
  ConditionNodeProps,
  DataProcessorNodeProps,
  WebhookNodeProps,
  TimerNodeProps,
  AIAgentNodeProps,
} from './nodes';

// 数据映射系统
export {
  DataMappingCanvas,
  DataMappingPreview,
  ExpressionEditor,
  DataConnector,
  MappingTemplates,
} from './datamapping';

export type {
  DataField,
  MappingRule,
  DataMappingConfig,
  TestResult,
  FieldVariable,
  ExpressionFunction,
  ConnectionConfig,
  DataPreview,
  MappingTemplate,
} from './datamapping';

// 执行引擎
export {
  WorkflowEngine,
  defaultWorkflowEngine,
  BaseNodeExecutor,
  TriggerNodeExecutor,
  AIAgentNodeExecutor,
  ConditionNodeExecutor,
  ActionNodeExecutor,
  DataProcessorNodeExecutor,
  nodeExecutors,
  ExecutionMonitor,
} from './execution';

export type {
  ExecutionStatus,
  NodeExecutionStatus,
  ExecutionContext,
  NodeExecutionResult,
  WorkflowNode,
  WorkflowEdge,
  WorkflowDefinition,
  EngineConfig,
  NodeExecutor,
} from './execution';

// 实时监控
export {
  RealTimeMonitor,
  Dashboard,
} from './monitoring';

// 性能优化
export {
  PerformanceOptimizer,
  WorkflowEngineOptimizer,
  CanvasOptimizer,
  PerformanceHub,
  PerformanceMonitor,
  PerformanceOptimizerUtils,
  usePerformanceMonitor,
  useMemoryMonitor,
  useFPSMonitor,
} from './optimization';

export type {
  EngineOptimizationConfig,
  CanvasOptimizationConfig,
} from './optimization';