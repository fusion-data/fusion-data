import { EventEmitter } from 'events';

// 执行状态类型
export type ExecutionStatus = 'idle' | 'running' | 'paused' | 'completed' | 'failed' | 'cancelled';

// 节点执行状态
export type NodeExecutionStatus = 'pending' | 'running' | 'completed' | 'failed' | 'skipped';

// 执行上下文接口
export interface ExecutionContext {
  workflowId: string;
  executionId: string;
  variables: Record<string, any>;
  startTime: Date;
  endTime?: Date;
  status: ExecutionStatus;
  error?: string;
  nodeResults: Record<string, any>;
}

// 节点执行结果接口
export interface NodeExecutionResult {
  nodeId: string;
  status: NodeExecutionStatus;
  startTime: Date;
  endTime?: Date;
  input?: any;
  output?: any;
  error?: string;
  duration?: number;
  metadata?: Record<string, any>;
}

// 工作流节点接口
export interface WorkflowNode {
  id: string;
  type: string;
  data: any;
  inputs: string[];
  outputs: string[];
  position: { x: number; y: number };
}

// 工作流连接接口
export interface WorkflowEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
}

// 工作流定义接口
export interface WorkflowDefinition {
  id: string;
  name: string;
  description?: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  variables?: Record<string, any>;
}

// 执行引擎配置接口
export interface EngineConfig {
  maxConcurrentNodes: number;
  timeout: number;
  retryAttempts: number;
  enableLogging: boolean;
  enableMetrics: boolean;
}

// 节点执行器接口
export interface NodeExecutor {
  execute(node: WorkflowNode, context: ExecutionContext): Promise<NodeExecutionResult>;
  validate(node: WorkflowNode): boolean;
  getNodeInfo(): {
    type: string;
    name: string;
    description: string;
    inputs: Array<{
      name: string;
      type: string;
      required: boolean;
    }>;
    outputs: Array<{
      name: string;
      type: string;
    }>;
  };
}

/**
 * 工作流执行引擎
 * 负责执行工作流定义，管理节点执行顺序和状态
 */
export class WorkflowEngine extends EventEmitter {
  private config: EngineConfig;
  private nodeExecutors: Map<string, NodeExecutor> = new Map();
  private activeExecutions: Map<string, ExecutionContext> = new Map();
  private nodeQueues: Map<string, string[]> = new Map(); // workflowId -> nodeId[]
  private runningNodes: Map<string, Set<string>> = new Map(); // workflowId -> Set of running nodeIds

  constructor(config: Partial<EngineConfig> = {}) {
    super();
    this.config = {
      maxConcurrentNodes: 5,
      timeout: 300000, // 5分钟
      retryAttempts: 3,
      enableLogging: true,
      enableMetrics: true,
      ...config,
    };
  }

  /**
   * 注册节点执行器
   */
  registerExecutor(nodeType: string, executor: NodeExecutor): void {
    this.nodeExecutors.set(nodeType, executor);
    this.emit('executor-registered', { nodeType, executor });
  }

  /**
   * 执行工作流
   */
  async execute(workflow: WorkflowDefinition, initialContext: Partial<ExecutionContext> = {}): Promise<ExecutionContext> {
    const executionId = this.generateExecutionId();
    const context: ExecutionContext = {
      workflowId: workflow.id,
      executionId,
      variables: { ...workflow.variables, ...initialContext.variables },
      startTime: new Date(),
      status: 'running',
      nodeResults: {},
    };

    this.activeExecutions.set(executionId, context);
    this.nodeQueues.set(workflow.id, this.buildExecutionQueue(workflow));
    this.runningNodes.set(workflow.id, new Set());

    this.emit('execution-started', { workflowId: workflow.id, executionId, context });

    try {
      await this.executeWorkflow(workflow, context);
      context.status = 'completed';
      context.endTime = new Date();
    } catch (error: any) {
      context.status = 'failed';
      context.error = error.message;
      context.endTime = new Date();
      this.emit('execution-failed', { workflowId: workflow.id, executionId, error });
    } finally {
      this.activeExecutions.delete(executionId);
      this.nodeQueues.delete(workflow.id);
      this.runningNodes.delete(workflow.id);
      this.emit('execution-completed', { workflowId: workflow.id, executionId, context });
    }

    return context;
  }

  /**
   * 暂停工作流执行
   */
  pause(executionId: string): void {
    const context = this.activeExecutions.get(executionId);
    if (context && context.status === 'running') {
      context.status = 'paused';
      this.emit('execution-paused', { executionId, context });
    }
  }

  /**
   * 恢复工作流执行
   */
  resume(executionId: string): void {
    const context = this.activeExecutions.get(executionId);
    if (context && context.status === 'paused') {
      context.status = 'running';
      this.emit('execution-resumed', { executionId, context });
    }
  }

  /**
   * 取消工作流执行
   */
  cancel(executionId: string): void {
    const context = this.activeExecutions.get(executionId);
    if (context && (context.status === 'running' || context.status === 'paused')) {
      context.status = 'cancelled';
      context.endTime = new Date();
      this.emit('execution-cancelled', { executionId, context });
    }
  }

  /**
   * 获取执行状态
   */
  getExecutionStatus(executionId: string): ExecutionContext | null {
    return this.activeExecutions.get(executionId) || null;
  }

  /**
   * 获取所有活跃执行
   */
  getActiveExecutions(): ExecutionContext[] {
    return Array.from(this.activeExecutions.values());
  }

  /**
   * 构建执行队列（拓扑排序）
   */
  private buildExecutionQueue(workflow: WorkflowDefinition): string[] {
    const { nodes, edges } = workflow;
    const adjacencyList: Map<string, string[]> = new Map();
    const inDegree: Map<string, number> = new Map();

    // 初始化图结构
    nodes.forEach(node => {
      adjacencyList.set(node.id, []);
      inDegree.set(node.id, 0);
    });

    // 构建邻接表和入度
    edges.forEach(edge => {
      const targets = adjacencyList.get(edge.source) || [];
      targets.push(edge.target);
      adjacencyList.set(edge.source, targets);
      inDegree.set(edge.target, (inDegree.get(edge.target) || 0) + 1);
    });

    // 拓扑排序
    const queue: string[] = [];
    const result: string[] = [];

    // 找到所有入度为0的节点
    inDegree.forEach((degree, nodeId) => {
      if (degree === 0) {
        queue.push(nodeId);
      }
    });

    while (queue.length > 0) {
      const current = queue.shift()!;
      result.push(current);

      // 减少邻接节点的入度
      const neighbors = adjacencyList.get(current) || [];
      neighbors.forEach(neighbor => {
        const newDegree = (inDegree.get(neighbor) || 0) - 1;
        inDegree.set(neighbor, newDegree);
        if (newDegree === 0) {
          queue.push(neighbor);
        }
      });
    }

    return result;
  }

  /**
   * 执行工作流
   */
  private async executeWorkflow(workflow: WorkflowDefinition, context: ExecutionContext): Promise<void> {
    const nodeQueue = this.nodeQueues.get(workflow.id)!;
    const runningNodes = this.runningNodes.get(workflow.id)!;

    while (nodeQueue.length > 0 && context.status === 'running') {
      // 获取可以执行的节点
      const readyNodes = this.getReadyNodes(workflow, nodeQueue, context, runningNodes);

      // 限制并发执行数量
      const nodesToExecute = readyNodes.slice(0, this.config.maxConcurrentNodes - runningNodes.size);

      // 并发执行节点
      const executionPromises = nodesToExecute.map(nodeId =>
        this.executeNode(workflow.nodes.find(n => n.id === nodeId)!, context)
      );

      if (executionPromises.length > 0) {
        await Promise.allSettled(executionPromises);
      } else if (runningNodes.size > 0) {
        // 等待正在运行的节点完成
        await new Promise(resolve => setTimeout(resolve, 100));
      } else {
        // 没有可执行的节点，退出循环
        break;
      }
    }
  }

  /**
   * 获取准备执行的节点
   */
  private getReadyNodes(
    workflow: WorkflowDefinition,
    nodeQueue: string[],
    context: ExecutionContext,
    runningNodes: Set<string>
  ): string[] {
    const readyNodes: string[] = [];
    const remainingNodes = [...nodeQueue];

    for (const nodeId of remainingNodes) {
      if (runningNodes.has(nodeId)) {
        continue;
      }

      const node = workflow.nodes.find(n => n.id === nodeId)!;
      const inputs = this.getNodeInputs(workflow, node);

      // 检查所有输入节点是否已完成
      const allInputsCompleted = inputs.every(inputId =>
        context.nodeResults[inputId]?.status === 'completed'
      );

      if (allInputsCompleted) {
        readyNodes.push(nodeId);
        // 从队列中移除
        const index = nodeQueue.indexOf(nodeId);
        if (index > -1) {
          nodeQueue.splice(index, 1);
        }
      }
    }

    return readyNodes;
  }

  /**
   * 获取节点的输入节点
   */
  private getNodeInputs(workflow: WorkflowDefinition, node: WorkflowNode): string[] {
    return workflow.edges
      .filter(edge => edge.target === node.id)
      .map(edge => edge.source);
  }

  /**
   * 执行单个节点
   */
  private async executeNode(node: WorkflowNode, context: ExecutionContext): Promise<void> {
    const runningNodes = this.runningNodes.get(context.workflowId)!;
    runningNodes.add(node.id);

    const startTime = new Date();
    const result: NodeExecutionResult = {
      nodeId: node.id,
      status: 'running',
      startTime,
      input: this.prepareNodeInput(node, context),
    };

    this.emit('node-started', { nodeId: node.id, context, result });

    try {
      const executor = this.nodeExecutors.get(node.type);
      if (!executor) {
        throw new Error(`No executor found for node type: ${node.type}`);
      }

      // 验证节点配置
      if (!executor.validate(node)) {
        throw new Error(`Invalid node configuration for: ${node.id}`);
      }

      // 执行节点
      const nodeResult = await Promise.race([
        executor.execute(node, context),
        this.createTimeoutPromise(this.config.timeout)
      ]);

      result.output = nodeResult.output;
      result.status = nodeResult.status;
      result.endTime = new Date();
      result.duration = result.endTime.getTime() - startTime.getTime();
      result.metadata = nodeResult.metadata;

      // 存储执行结果
      context.nodeResults[node.id] = result;

      this.emit('node-completed', { nodeId: node.id, context, result });

    } catch (error: any) {
      result.status = 'failed';
      result.error = error.message;
      result.endTime = new Date();
      result.duration = result.endTime.getTime() - startTime.getTime();

      context.nodeResults[node.id] = result;

      this.emit('node-failed', { nodeId: node.id, context, result, error });

      // 根据配置决定是否继续执行
      if (this.shouldStopOnError(node, error)) {
        context.status = 'failed';
        context.error = `Node ${node.id} failed: ${error.message}`;
        throw error;
      }
    } finally {
      runningNodes.delete(node.id);
    }
  }

  /**
   * 准备节点输入数据
   */
  private prepareNodeInput(node: WorkflowNode, context: ExecutionContext): any {
    const input: any = {};

    // 从变量中获取输入
    if (node.data.inputs) {
      Object.entries(node.data.inputs).forEach(([key, value]) => {
        if (typeof value === 'string' && value.startsWith('$')) {
          const varName = value.substring(1);
          input[key] = context.variables[varName];
        } else {
          input[key] = value;
        }
      });
    }

    // 从上游节点获取输出
    // 这里需要根据工作流的连接关系来获取上游节点的输出

    return input;
  }

  /**
   * 创建超时Promise
   */
  private createTimeoutPromise(timeout: number): Promise<never> {
    return new Promise((_, reject) => {
      setTimeout(() => reject(new Error('Node execution timeout')), timeout);
    });
  }

  /**
   * 判断是否应该在错误时停止执行
   */
  private shouldStopOnError(node: WorkflowNode, error: any): boolean {
    // 根据节点配置决定错误处理策略
    return node.data.stopOnError !== false;
  }

  /**
   * 生成执行ID
   */
  private generateExecutionId(): string {
    return `exec_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 获取引擎统计信息
   */
  getStats() {
    return {
      activeExecutions: this.activeExecutions.size,
      registeredExecutors: this.nodeExecutors.size,
      config: this.config,
    };
  }
}

// 创建默认的工作流引擎实例
export const defaultWorkflowEngine = new WorkflowEngine();