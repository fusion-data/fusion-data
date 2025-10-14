import React from 'react';
import { Position, XYPosition } from '@xyflow/react';

// 节点基础属性
export interface BaseNodeProps {
  data: {
    label: string;
    description?: string;
    type: string;
    status?: 'idle' | 'running' | 'success' | 'error';
    config?: Record<string, any>;
    icon?: React.ReactNode;
  };
  selected?: boolean;
  id: string;
}

// 节点连接点配置
export interface HandleConfig {
  type: 'source' | 'target';
  position: Position;
  style?: React.CSSProperties;
}

// 节点样式配置
export interface NodeStyleConfig {
  backgroundColor?: string;
  borderColor?: string;
  borderWidth?: number;
  borderRadius?: number;
  padding?: string;
  boxShadow?: string;
}

// 输入输出数据
export interface NodeData {
  inputs: NodePort[];
  outputs: NodePort[];
}

// 节点端口
export interface NodePort {
  id: string;
  name: string;
  type: string;
  description?: string;
  required?: boolean;
}

// 节点状态
export enum NodeStatus {
  IDLE = 'idle',
  RUNNING = 'running',
  SUCCESS = 'success',
  ERROR = 'error',
}

// 节点类型
export enum NodeType {
  TRIGGER = 'trigger',
  ACTION = 'action',
  CONDITION = 'condition',
  DATA_PROCESSOR = 'dataProcessor',
  WEBHOOK = 'webhook',
  TIMER = 'timer',
  API_CALL = 'apiCall',
  CODE_EXECUTION = 'codeExecution',
  DATABASE = 'database',
  EMAIL = 'email',
  FILE_HANDLER = 'fileHandler',
  AI_AGENT = 'aiAgent',
}

// 节点组件属性
export interface CustomNodeProps {
  id: string;
  type: NodeType;
  label: string;
  description?: string;
  icon?: React.ReactNode;
  status?: NodeStatus;
  data?: any;
  selected?: boolean;
  dragging?: boolean;
  handles?: HandleConfig[];
  style?: NodeStyleConfig;
  onEdit?: () => void;
  onDelete?: () => void;
  onDuplicate?: () => void;
}

// 节点配置（用于工厂创建）
export interface NodeConfig {
  id: string;
  type: string;
  data: {
    label: string;
    description?: string;
    type: string;
    status?: 'idle' | 'running' | 'success' | 'error';
    config?: Record<string, any>;
    icon?: React.ReactNode;
  };
  position: XYPosition;
  style?: React.CSSProperties;
}

// 节点类型配置（工厂配置）
export interface NodeTypeConfig {
  category: string;
  displayName: string;
  description: string;
  icon: string;
  color: string;
  backgroundColor: string;
  borderColor: string;
  allowedConnections: Array<'target' | 'source'>;
  inputs: NodePort[];
  outputs: NodePort[];
}

// 节点模板配置
export interface NodeTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  type: string;
  icon: string;
  tags: string[];
  config: Partial<NodeConfig>;
  preview?: string;
}