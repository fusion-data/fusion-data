import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { Node } from '@xyflow/react';
import { nodeFactory } from './NodeFactory';
import { nodeRegistry } from './NodeRegistry';
import { NodeUtils } from './NodeUtils';

interface NodeContextType {
  // 节点选择状态
  selectedNodeId: string | null;
  hoveredNodeId: string | null;

  // 操作
  selectNode: (nodeId: string | null) => void;
  hoverNode: (nodeId: string | null) => void;

  // 节点创建
  createNode: (type: string, position?: { x: number; y: number }) => Node;
  duplicateNode: (nodeId: string) => Node | null;
  deleteNode: (nodeId: string) => void;

  // 节点验证
  validateConnection: (sourceNode: Node, targetNode: Node) => boolean;

  // 节点信息
  getNodeConfig: (nodeId: string) => any;
  getNodeInfo: (nodeId: string) => any;

  // 工具函数
  generateNodeId: (type: string) => string;
  calculateNodePosition: (existingNodes: Node[], preferredPosition?: { x: number; y: number }) => { x: number; y: number };
}

const NodeContext = createContext<NodeContextType | undefined>(undefined);

interface NodeProviderProps {
  children: ReactNode;
  existingNodes?: Node[];
}

export const NodeProvider: React.FC<NodeProviderProps> = ({
  children,
  existingNodes = []
}) => {
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null);

  // 选择节点
  const selectNode = useCallback((nodeId: string | null) => {
    setSelectedNodeId(nodeId);
  }, []);

  // 悬停节点
  const hoverNode = useCallback((nodeId: string | null) => {
    setHoveredNodeId(nodeId);
  }, []);

  // 创建新节点
  const createNode = useCallback((type: string, position?: { x: number; y: number }) => {
    const calculatedPosition = NodeUtils.calculateNodePosition(existingNodes, position);
    const nodeConfig = nodeFactory.createNode(type, { position: calculatedPosition });

    // 注册到节点注册表
    nodeRegistry.registerNode(nodeConfig);

    // 转换为 React Flow 节点格式
    const node: Node = {
      id: nodeConfig.id,
      type: nodeConfig.type,
      position: nodeConfig.position,
      data: nodeConfig.data,
      style: nodeConfig.style,
    };

    return node;
  }, [existingNodes]);

  // 复制节点
  const duplicateNode = useCallback((nodeId: string) => {
    const existingNode = nodeRegistry.getNode(nodeId);
    if (!existingNode) {
      return null;
    }

    const clonedConfig = NodeUtils.cloneNodeConfig(existingNode);
    nodeRegistry.registerNode(clonedConfig);

    const node: Node = {
      id: clonedConfig.id,
      type: clonedConfig.type,
      position: clonedConfig.position,
      data: clonedConfig.data,
      style: clonedConfig.style,
    };

    return node;
  }, []);

  // 删除节点
  const deleteNode = useCallback((nodeId: string) => {
    nodeRegistry.unregisterNode(nodeId);
    if (selectedNodeId === nodeId) {
      setSelectedNodeId(null);
    }
    if (hoveredNodeId === nodeId) {
      setHoveredNodeId(null);
    }
  }, [selectedNodeId, hoveredNodeId]);

  // 验证连接
  const validateConnection = useCallback((sourceNode: Node, targetNode: Node) => {
    return NodeUtils.canConnect(sourceNode, targetNode);
  }, []);

  // 获取节点配置
  const getNodeConfig = useCallback((nodeId: string) => {
    return nodeRegistry.getNode(nodeId);
  }, []);

  // 获取节点信息
  const getNodeInfo = useCallback((nodeId: string) => {
    const node = nodeRegistry.getNode(nodeId);
    if (!node) {
      return null;
    }

    const typeConfig = nodeFactory.getNodeConfig(node.type);
    return {
      ...node,
      typeConfig,
      displayName: NodeUtils.getNodeDisplayName(node as any),
      description: NodeUtils.getNodeDescription(node as any),
      status: NodeUtils.getNodeStatus(node as any),
      inputs: NodeUtils.getNodeInputs(node as any),
      outputs: NodeUtils.getNodeOutputs(node as any),
    };
  }, []);

  // 生成节点 ID
  const generateNodeId = useCallback((type: string) => {
    return NodeUtils.generateNodeId(type);
  }, []);

  // 计算节点位置
  const calculateNodePosition = useCallback((
    existingNodes: Node[],
    preferredPosition?: { x: number; y: number }
  ) => {
    return NodeUtils.calculateNodePosition(existingNodes, preferredPosition);
  }, []);

  const value: NodeContextType = {
    // 状态
    selectedNodeId,
    hoveredNodeId,

    // 操作
    selectNode,
    hoverNode,
    createNode,
    duplicateNode,
    deleteNode,
    validateConnection,
    getNodeConfig,
    getNodeInfo,
    generateNodeId,
    calculateNodePosition,
  };

  return (
    <NodeContext.Provider value={value}>
      {children}
    </NodeContext.Provider>
  );
};

export const useNodeContext = (): NodeContextType => {
  const context = useContext(NodeContext);
  if (context === undefined) {
    throw new Error('useNodeContext must be used within a NodeProvider');
  }
  return context;
};

export default NodeContext;