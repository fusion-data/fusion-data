import React from 'react';
import { NodeTypes } from '@xyflow/react';
import TriggerNode from './TriggerNode';
import ActionNode from './ActionNode';
import ConditionNode from './ConditionNode';
import DataProcessorNode from './DataProcessorNode';
import WebhookNode from './WebhookNode';
import TimerNode from './TimerNode';
import AIAgentNode from './AIAgentNode';
import type { NodeConfig, NodeTypeConfig } from './types';

/**
 * 节点工厂类 - 负责创建和管理所有类型的节点
 */
export class NodeFactory {
  private static instance: NodeFactory;
  private nodeTypes: NodeTypes;
  private nodeConfigs: Map<string, NodeTypeConfig>;

  private constructor() {
    this.nodeTypes = {};
    this.nodeConfigs = new Map();
    this.initializeNodes();
  }

  /**
   * 获取单例实例
   */
  public static getInstance(): NodeFactory {
    if (!NodeFactory.instance) {
      NodeFactory.instance = new NodeFactory();
    }
    return NodeFactory.instance;
  }

  /**
   * 初始化所有节点类型
   */
  private initializeNodes(): void {
    // 注册基础节点类型
    this.registerNodeType('trigger', TriggerNode, {
      category: 'triggers',
      displayName: '触发器',
      description: '工作流的起始触发点',
      icon: 'PlayCircleOutlined',
      color: '#52c41a',
      backgroundColor: '#f6ffed',
      borderColor: '#b7eb8f',
      allowedConnections: ['source'],
      inputs: [],
      outputs: [
        {
          id: 'output',
          name: '输出',
          type: 'any',
          description: '触发器产生的数据输出',
          required: false,
        },
      ],
    });

    this.registerNodeType('action', ActionNode, {
      category: 'actions',
      displayName: '动作',
      description: '执行具体操作的任务节点',
      icon: 'ApiOutlined',
      color: '#1890ff',
      backgroundColor: '#f0f9ff',
      borderColor: '#91d5ff',
      allowedConnections: ['target', 'source'],
      inputs: [
        {
          id: 'input',
          name: '输入',
          type: 'any',
          description: '接收来自上游节点的数据',
          required: false,
        },
      ],
      outputs: [
        {
          id: 'output',
          name: '输出',
          type: 'any',
          description: '执行结果数据输出',
          required: false,
        },
      ],
    });

    this.registerNodeType('condition', ConditionNode, {
      category: 'control',
      displayName: '条件判断',
      description: '根据条件控制工作流分支',
      icon: 'BranchesOutlined',
      color: '#fa8c16',
      backgroundColor: '#fff7e6',
      borderColor: '#ffd591',
      allowedConnections: ['target', 'source'],
      inputs: [
        {
          id: 'input',
          name: '输入',
          type: 'any',
          description: '需要判断的数据',
          required: true,
        },
      ],
      outputs: [
        {
          id: 'true',
          name: '真分支',
          type: 'any',
          description: '条件为真时的输出',
          required: false,
        },
        {
          id: 'false',
          name: '假分支',
          type: 'any',
          description: '条件为假时的输出',
          required: false,
        },
      ],
    });

    this.registerNodeType('dataProcessor', DataProcessorNode, {
      category: 'data',
      displayName: '数据处理器',
      description: '转换、过滤或处理数据',
      icon: 'DatabaseOutlined',
      color: '#722ed1',
      backgroundColor: '#f9f0ff',
      borderColor: '#d3adf7',
      allowedConnections: ['target', 'source'],
      inputs: [
        {
          id: 'input',
          name: '输入',
          type: 'any',
          description: '需要处理的数据',
          required: true,
        },
      ],
      outputs: [
        {
          id: 'output',
          name: '输出',
          type: 'any',
          description: '处理后的数据',
          required: true,
        },
      ],
    });

    this.registerNodeType('webhook', WebhookNode, {
      category: 'triggers',
      displayName: 'Webhook',
      description: '通过 HTTP 请求触发工作流',
      icon: 'ApiOutlined',
      color: '#eb2f96',
      backgroundColor: '#fff0f6',
      borderColor: '#ffadd2',
      allowedConnections: ['source'],
      inputs: [],
      outputs: [
        {
          id: 'output',
          name: '请求数据',
          type: 'object',
          description: 'HTTP 请求的完整数据',
          required: true,
        },
      ],
    });

    this.registerNodeType('timer', TimerNode, {
      category: 'triggers',
      displayName: '定时器',
      description: '按时间计划触发工作流',
      icon: 'ClockCircleOutlined',
      color: '#13c2c2',
      backgroundColor: '#e6fffb',
      borderColor: '#87e8de',
      allowedConnections: ['source'],
      inputs: [],
      outputs: [
        {
          id: 'output',
          name: '触发时间',
          type: 'object',
          description: '定时器触发时的信息',
          required: true,
        },
      ],
    });

    this.registerNodeType('aiAgent', AIAgentNode, {
      category: 'ai',
      displayName: 'AI 智能体',
      description: '使用 AI 模型处理任务',
      icon: 'RobotOutlined',
      color: '#1890ff',
      backgroundColor: '#f0f9ff',
      borderColor: '#91d5ff',
      allowedConnections: ['target', 'source'],
      inputs: [
        {
          id: 'input',
          name: '输入',
          type: 'string',
          description: '发送给 AI 的提示或问题',
          required: true,
        },
      ],
      outputs: [
        {
          id: 'output',
          name: 'AI 响应',
          type: 'object',
          description: 'AI 生成的响应内容',
          required: true,
        },
      ],
    });
  }

  /**
   * 注册新的节点类型
   */
  public registerNodeType(
    type: string,
    component: React.ComponentType<any>,
    config: NodeTypeConfig
  ): void {
    this.nodeTypes[type] = component;
    this.nodeConfigs.set(type, config);
  }

  /**
   * 获取所有节点类型
   */
  public getNodeTypes(): NodeTypes {
    return { ...this.nodeTypes };
  }

  /**
   * 获取节点配置
   */
  public getNodeConfig(type: string): NodeTypeConfig | undefined {
    return this.nodeConfigs.get(type);
  }

  /**
   * 获取所有节点配置
   */
  public getAllNodeConfigs(): NodeTypeConfig[] {
    return Array.from(this.nodeConfigs.values());
  }

  /**
   * 根据分类获取节点配置
   */
  public getNodeConfigsByCategory(category: string): NodeTypeConfig[] {
    return Array.from(this.nodeConfigs.values()).filter(
      config => config.category === category
    );
  }

  /**
   * 检查节点类型是否存在
   */
  public hasNodeType(type: string): boolean {
    return this.nodeConfigs.has(type);
  }

  /**
   * 创建新节点
   */
  public createNode(type: string, config: Partial<NodeConfig> = {}): NodeConfig {
    const nodeTypeConfig = this.nodeConfigs.get(type);
    if (!nodeTypeConfig) {
      throw new Error(`Unknown node type: ${type}`);
    }

    const now = Date.now();
    const nodeId = `${type}_${now}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      id: nodeId,
      type,
      data: {
        label: nodeTypeConfig.displayName,
        description: nodeTypeConfig.description,
        type,
        status: 'idle',
        config: config.data || {},
        icon: nodeTypeConfig.icon,
      },
      position: config.position || { x: 0, y: 0 },
      style: {
        backgroundColor: nodeTypeConfig.backgroundColor,
        borderColor: nodeTypeConfig.borderColor,
        color: nodeTypeConfig.color,
        ...config.style,
      },
      ...config,
    };
  }

  /**
   * 获取节点分类
   */
  public getCategories(): Array<{ key: string; label: string; description: string }> {
    const categories = new Map<string, { label: string; description: string }>();

    this.nodeConfigs.forEach(config => {
      if (!categories.has(config.category)) {
        const categoryInfo = this.getCategoryInfo(config.category);
        categories.set(config.category, categoryInfo);
      }
    });

    return Array.from(categories.entries()).map(([key, info]) => ({
      key,
      ...info,
    }));
  }

  /**
   * 获取分类信息
   */
  private getCategoryInfo(category: string): { label: string; description: string } {
    const categoryMap: Record<string, { label: string; description: string }> = {
      triggers: {
        label: '触发器',
        description: '工作流的起始节点，用于启动工作流执行',
      },
      actions: {
        label: '动作',
        description: '执行具体操作的任务节点',
      },
      control: {
        label: '控制流',
        description: '控制工作流的执行路径和逻辑',
      },
      data: {
        label: '数据处理',
        description: '处理、转换和操作数据的节点',
      },
      ai: {
        label: 'AI 智能体',
        description: '集成 AI 功能的智能节点',
      },
      integration: {
        label: '集成服务',
        description: '与外部服务和系统集成的节点',
      },
    };

    return (
      categoryMap[category] || {
        label: category,
        description: `${category} 类型的节点`,
      }
    );
  }
}

// 导出单例实例
export const nodeFactory = NodeFactory.getInstance();