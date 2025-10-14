import { Viewport } from '@xyflow/react';

export interface WorkflowNode {
  id: string;
  type: string;
  name: string;
  description?: string;
  data: any;
  position: { x: number; y: number };
}

export interface WorkflowEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
  type?: string;
  animated?: boolean;
  data?: any;
}

export interface Workflow {
  id: string;
  name: string;
  description?: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  metadata: WorkflowMetadata;
}

export interface WorkflowMetadata {
  createdAt: string;
  updatedAt: string;
  version: string;
  tags?: string[];
  category?: string;
}

export interface ViewportState extends Viewport {
  x: number;
  y: number;
  zoom: number;
}

export interface WorkflowExecution {
  id: string;
  workflowId: string;
  startTime: string;
  endTime?: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  input?: any;
  output?: any;
  error?: string;
  durationMs?: number;
  nodeExecutions?: NodeExecution[];
}

export interface NodeExecution {
  id: string;
  nodeId: string;
  startTime: string;
  endTime?: string;
  status: 'running' | 'completed' | 'failed' | 'skipped';
  input?: any;
  output?: any;
  error?: string;
  durationMs?: number;
  retryCount?: number;
}