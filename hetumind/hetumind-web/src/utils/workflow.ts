import { v4 as uuidv4 } from 'uuid';
import { WorkflowNode, WorkflowEdge, Workflow } from '@/types/workflow';

/**
 * 生成唯一 ID
 */
export const generateId = (): string => {
  return uuidv4();
};

/**
 * 创建新的工作流
 */
export const createWorkflow = (name: string, description?: string): Workflow => {
  return {
    id: generateId(),
    name,
    description,
    nodes: [],
    edges: [],
    metadata: {
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      version: '1.0.0',
    },
  };
};

/**
 * 创建新的节点
 */
export const createNode = (
  type: string,
  name: string,
  position: { x: number; y: number },
  data?: any
): WorkflowNode => {
  return {
    id: generateId(),
    type,
    name,
    position,
    data: data || {},
  };
};

/**
 * 创建新的连接
 */
export const createEdge = (
  source: string,
  target: string,
  sourceHandle?: string,
  targetHandle?: string
): WorkflowEdge => {
  return {
    id: generateId(),
    source,
    target,
    sourceHandle,
    targetHandle,
    type: 'smoothstep',
    animated: true,
  };
};

/**
 * 验证工作流
 */
export const validateWorkflow = (workflow: Workflow): { valid: boolean; errors: string[] } => {
  const errors: string[] = [];

  if (!workflow.name || workflow.name.trim() === '') {
    errors.push('工作流名称不能为空');
  }

  if (workflow.nodes.length === 0) {
    errors.push('工作流至少需要一个节点');
  }

  // 检查连接的有效性
  workflow.edges.forEach(edge => {
    const sourceNode = workflow.nodes.find(node => node.id === edge.source);
    const targetNode = workflow.nodes.find(node => node.id === edge.target);

    if (!sourceNode) {
      errors.push(`连接 ${edge.id} 的源节点 ${edge.source} 不存在`);
    }

    if (!targetNode) {
      errors.push(`连接 ${edge.id} 的目标节点 ${edge.target} 不存在`);
    }
  });

  return {
    valid: errors.length === 0,
    errors,
  };
};

/**
 * 查找孤立节点（没有连接的节点）
 */
export const findIsolatedNodes = (workflow: Workflow): string[] => {
  const connectedNodeIds = new Set<string>();

  workflow.edges.forEach(edge => {
    connectedNodeIds.add(edge.source);
    connectedNodeIds.add(edge.target);
  });

  return workflow.nodes
    .filter(node => !connectedNodeIds.has(node.id))
    .map(node => node.id);
};

/**
 * 查找起始节点（没有输入连接的节点）
 */
export const findStartNodes = (workflow: Workflow): string[] => {
  const targetNodeIds = new Set(workflow.edges.map(edge => edge.target));

  return workflow.nodes
    .filter(node => !targetNodeIds.has(node.id))
    .map(node => node.id);
};

/**
 * 查找结束节点（没有输出连接的节点）
 */
export const findEndNodes = (workflow: Workflow): string[] => {
  const sourceNodeIds = new Set(workflow.edges.map(edge => edge.source));

  return workflow.nodes
    .filter(node => !sourceNodeIds.has(node.id))
    .map(node => node.id);
};

/**
 * 检查工作流是否存在循环
 */
export const hasCycle = (workflow: Workflow): boolean => {
  const visited = new Set<string>();
  const recursionStack = new Set<string>();

  const dfs = (nodeId: string): boolean => {
    if (recursionStack.has(nodeId)) {
      return true; // 发现循环
    }

    if (visited.has(nodeId)) {
      return false;
    }

    visited.add(nodeId);
    recursionStack.add(nodeId);

    // 查找当前节点的所有相邻节点
    const adjacentNodes = workflow.edges
      .filter(edge => edge.source === nodeId)
      .map(edge => edge.target);

    for (const adjacentNodeId of adjacentNodes) {
      if (dfs(adjacentNodeId)) {
        return true;
      }
    }

    recursionStack.delete(nodeId);
    return false;
  };

  // 从每个节点开始 DFS
  for (const node of workflow.nodes) {
    if (!visited.has(node.id)) {
      if (dfs(node.id)) {
        return true;
      }
    }
  }

  return false;
};