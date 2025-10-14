import { Node, XYPosition } from '@xyflow/react';
import { NodeConfig } from './types';
import { NodeFactory } from './NodeFactory';

/**
 * 节点工具类 - 提供节点操作的通用方法
 */
export class NodeUtils {
  /**
   * 验证节点配置是否有效
   */
  public static validateNodeConfig(config: Partial<NodeConfig>): boolean {
    if (!config.type) {
      return false;
    }

    const nodeFactoryInstance = NodeFactory.getInstance();
    if (!nodeFactoryInstance.hasNodeType(config.type)) {
      return false;
    }

    return true;
  }

  /**
   * 计算节点位置，避免重叠
   */
  public static calculateNodePosition(
    existingNodes: Node[],
    preferredPosition?: XYPosition,
    spacing: { x: number; y: number } = { x: 250, y: 100 }
  ): XYPosition {
    if (!preferredPosition) {
      // 如果没有首选位置，放置在画布中心
      return { x: 100, y: 100 };
    }

    // 检查是否有重叠
    const hasOverlap = existingNodes.some(node => {
      const distance = Math.sqrt(
        Math.pow(node.position.x - preferredPosition.x, 2) +
        Math.pow(node.position.y - preferredPosition.y, 2)
      );
      return distance < spacing.x;
    });

    if (!hasOverlap) {
      return preferredPosition;
    }

    // 找到最合适的空白位置
    let bestPosition = preferredPosition;
    let minDistance = Infinity;

    for (let x = 0; x < 5; x++) {
      for (let y = 0; y < 5; y++) {
        const testPosition = {
          x: preferredPosition.x + x * spacing.x,
          y: preferredPosition.y + y * spacing.y,
        };

        const totalDistance = existingNodes.reduce((sum, node) => {
          return (
            sum +
            Math.sqrt(
              Math.pow(node.position.x - testPosition.x, 2) +
              Math.pow(node.position.y - testPosition.y, 2)
            )
          );
        }, 0);

        if (totalDistance < minDistance) {
          minDistance = totalDistance;
          bestPosition = testPosition;
        }
      }
    }

    return bestPosition;
  }

  /**
   * 检查两个节点是否可以连接
   */
  public static canConnect(
    _sourceNode: Node,
    _targetNode: Node,
    _sourceHandle?: string,
    _targetHandle?: string
  ): boolean {
    const nodeFactoryInstance = NodeFactory.getInstance();
    const sourceConfig = nodeFactoryInstance.getNodeConfig(_sourceNode.type || '');
    const targetConfig = nodeFactoryInstance.getNodeConfig(_targetNode.type || '');

    if (!sourceConfig || !targetConfig) {
      return false;
    }

    // 检查源节点是否允许作为输出
    if (!sourceConfig?.allowedConnections.includes('source')) {
      return false;
    }

    // 检查目标节点是否允许作为输入
    if (!targetConfig?.allowedConnections.includes('target')) {
      return false;
    }

    // 检查是否存在循环依赖
    if (this.wouldCreateCycle(_sourceNode, _targetNode)) {
      return false;
    }

    return true;
  }

  /**
   * 检查连接是否会创建循环
   */
  private static wouldCreateCycle(_sourceNode: Node, _targetNode: Node): boolean {
    // 简单的循环检测：如果目标节点已经在源节点的下游，则会产生循环
    // 这里可以实现更复杂的图算法
    return false; // 暂时简化实现
  }

  /**
   * 生成唯一的节点 ID
   */
  public static generateNodeId(type: string): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substr(2, 9);
    return `${type}_${timestamp}_${random}`;
  }

  /**
   * 克隆节点配置
   */
  public static cloneNodeConfig(
    originalConfig: NodeConfig,
    position?: XYPosition
  ): NodeConfig {
    const clonedConfig: NodeConfig = {
      ...originalConfig,
      id: this.generateNodeId(originalConfig.type),
      position: position || {
        x: originalConfig.position.x + 50,
        y: originalConfig.position.y + 50,
      },
      data: {
        ...originalConfig.data,
        label: `${originalConfig.data.label} (副本)`,
      },
    };

    return clonedConfig;
  }

  /**
   * 序列化节点配置为 JSON
   */
  public static serializeNode(config: NodeConfig): string {
    return JSON.stringify(config, null, 2);
  }

  /**
   * 从 JSON 反序列化节点配置
   */
  public static deserializeNode(jsonString: string): NodeConfig | null {
    try {
      const config = JSON.parse(jsonString);
      if (this.validateNodeConfig(config)) {
        return config;
      }
      return null;
    } catch (error) {
      console.error('Failed to deserialize node config:', error);
      return null;
    }
  }

  /**
   * 获取节点的显示名称
   */
  public static getNodeDisplayName(node: Node): string {
    return (node.data as any)?.label || node.type || 'Unknown Node';
  }

  /**
   * 获取节点的描述信息
   */
  public static getNodeDescription(node: Node): string {
    return (node.data as any)?.description || '';
  }

  /**
   * 获取节点的状态
   */
  public static getNodeStatus(node: Node): 'idle' | 'running' | 'success' | 'error' {
    return (node.data as any)?.status || 'idle';
  }

  /**
   * 设置节点状态
   */
  public static setNodeStatus(
    node: Node,
    status: 'idle' | 'running' | 'success' | 'error'
  ): Node {
    return {
      ...node,
      data: {
        ...node.data,
        status,
      },
    };
  }

  /**
   * 检查节点是否为触发器节点
   */
  public static isTriggerNode(node: Node): boolean {
    const nodeFactoryInstance = NodeFactory.getInstance();
    const config = nodeFactoryInstance.getNodeConfig(node.type || '');
    return config?.category === 'triggers' || false;
  }

  /**
   * 检查节点是否为动作节点
   */
  public static isActionNode(node: Node): boolean {
    const nodeFactoryInstance = NodeFactory.getInstance();
    const config = nodeFactoryInstance.getNodeConfig(node.type || '');
    return config?.category === 'actions' || false;
  }

  /**
   * 获取节点的输入端口
   */
  public static getNodeInputs(node: Node): any[] {
    const nodeFactoryInstance = NodeFactory.getInstance();
    const config = nodeFactoryInstance.getNodeConfig(node.type || '');
    return config?.inputs || [];
  }

  /**
   * 获取节点的输出端口
   */
  public static getNodeOutputs(node: Node): any[] {
    const nodeFactoryInstance = NodeFactory.getInstance();
    const config = nodeFactoryInstance.getNodeConfig(node.type || '');
    return config?.outputs || [];
  }

  /**
   * 格式化节点大小
   */
  public static formatNodeSize(nodeType: string): { width: number; height: number } {
    // 根据节点类型返回不同的大小
    switch (nodeType) {
      case 'trigger':
      case 'webhook':
      case 'timer':
        return { width: 180, height: 80 };
      case 'condition':
        return { width: 200, height: 120 };
      case 'dataProcessor':
      case 'aiAgent':
        return { width: 220, height: 100 };
      case 'action':
      default:
        return { width: 200, height: 90 };
    }
  }
}