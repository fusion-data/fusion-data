// NodeRegistry - manages node instances, unused import removed
import { NodeConfig } from './types';

/**
 * 节点注册表 - 提供全局节点管理功能
 */
export class NodeRegistry {
  private static instance: NodeRegistry;
  private registeredNodes: Map<string, NodeConfig>;

  private constructor() {
    this.registeredNodes = new Map();
  }

  /**
   * 获取单例实例
   */
  public static getInstance(): NodeRegistry {
    if (!NodeRegistry.instance) {
      NodeRegistry.instance = new NodeRegistry();
    }
    return NodeRegistry.instance;
  }

  /**
   * 注册节点实例
   */
  public registerNode(node: NodeConfig): void {
    this.registeredNodes.set(node.id, node);
  }

  /**
   * 注销节点实例
   */
  public unregisterNode(nodeId: string): void {
    this.registeredNodes.delete(nodeId);
  }

  /**
   * 获取节点实例
   */
  public getNode(nodeId: string): NodeConfig | undefined {
    return this.registeredNodes.get(nodeId);
  }

  /**
   * 更新节点实例
   */
  public updateNode(nodeId: string, updates: Partial<NodeConfig>): void {
    const existingNode = this.registeredNodes.get(nodeId);
    if (existingNode) {
      this.registeredNodes.set(nodeId, {
        ...existingNode,
        ...updates,
        id: nodeId, // 确保 ID 不被覆盖
      });
    }
  }

  /**
   * 获取所有节点实例
   */
  public getAllNodes(): NodeConfig[] {
    return Array.from(this.registeredNodes.values());
  }

  /**
   * 根据类型获取节点实例
   */
  public getNodesByType(type: string): NodeConfig[] {
    return Array.from(this.registeredNodes.values()).filter(
      node => node.type === type
    );
  }

  /**
   * 清空所有节点实例
   */
  public clear(): void {
    this.registeredNodes.clear();
  }

  /**
   * 获取节点数量
   */
  public getNodeCount(): number {
    return this.registeredNodes.size;
  }

  /**
   * 检查节点是否存在
   */
  public hasNode(nodeId: string): boolean {
    return this.registeredNodes.has(nodeId);
  }
}

// 导出单例实例
export const nodeRegistry = NodeRegistry.getInstance();