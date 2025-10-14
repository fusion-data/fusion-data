import { NodeExecutor, NodeExecutionResult, WorkflowNode, ExecutionContext } from './WorkflowEngine';

/**
 * 抽象节点执行器基类
 */
export abstract class BaseNodeExecutor implements NodeExecutor {
  abstract execute(node: WorkflowNode, context: ExecutionContext): Promise<NodeExecutionResult>;
  abstract validate(node: WorkflowNode): boolean;
  abstract getNodeInfo(): {
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

  protected createResult(nodeId: string, status: 'completed' | 'failed', data?: any, error?: string): NodeExecutionResult {
    return {
      nodeId,
      status,
      startTime: new Date(),
      endTime: new Date(),
      output: data,
      error,
      duration: 0,
    };
  }
}

/**
 * 触发器节点执行器
 */
export class TriggerNodeExecutor extends BaseNodeExecutor {
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<NodeExecutionResult> {
    const { triggerType, config } = node.data;

    try {
      let output: any = {};

      switch (triggerType) {
        case 'manual':
          output = { triggered: true, timestamp: new Date().toISOString() };
          break;
        case 'schedule':
          output = {
            triggered: true,
            timestamp: new Date().toISOString(),
            schedule: config?.cronExpression
          };
          break;
        case 'webhook':
          output = {
            triggered: true,
            timestamp: new Date().toISOString(),
            webhook: config?.webhookUrl
          };
          break;
        default:
          throw new Error(`Unsupported trigger type: ${triggerType}`);
      }

      return this.createResult(node.id, 'completed', output);
    } catch (error: any) {
      return this.createResult(node.id, 'failed', undefined, error.message);
    }
  }

  validate(node: WorkflowNode): boolean {
    const { triggerType, config } = node.data;

    if (!triggerType) return false;

    if (triggerType === 'schedule' && !config?.cronExpression) return false;
    if (triggerType === 'webhook' && !config?.webhookUrl) return false;

    return true;
  }

  getNodeInfo() {
    return {
      type: 'trigger',
      name: '触发器',
      description: '工作流触发器，支持手动、定时和Webhook触发',
      inputs: [],
      outputs: [
        { name: 'triggered', type: 'boolean' },
        { name: 'timestamp', type: 'string' },
      ],
    };
  }
}

/**
 * AI Agent节点执行器
 */
export class AIAgentNodeExecutor extends BaseNodeExecutor {
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<NodeExecutionResult> {
    const { config, input } = node.data;

    try {
      // 模拟AI Agent执行
      await new Promise(resolve => setTimeout(resolve, 1000 + Math.random() * 2000));

      let output: any = {};

      switch (config?.agentType) {
        case 'chat':
          output = {
            response: `这是对"${input?.message || '默认消息'}"的AI回复`,
            model: config?.model || 'gpt-3.5-turbo',
            tokens: Math.floor(100 + Math.random() * 500),
          };
          break;
        case 'completion':
          output = {
            text: `这是基于提示"${input?.prompt || '默认提示'}"生成的文本`,
            model: config?.model || 'gpt-3.5-turbo',
            tokens: Math.floor(200 + Math.random() * 800),
          };
          break;
        case 'embedding':
          output = {
            embedding: Array.from({ length: 1536 }, () => Math.random()),
            model: config?.model || 'text-embedding-ada-002',
            dimensions: 1536,
          };
          break;
        default:
          throw new Error(`Unsupported agent type: ${config?.agentType}`);
      }

      // 添加元数据
      const metadata = {
        temperature: config?.temperature || 0.7,
        maxTokens: config?.maxTokens || 1024,
        responseTime: 1000 + Math.random() * 2000,
      };

      return {
        nodeId: node.id,
        status: 'completed',
        startTime: new Date(),
        endTime: new Date(),
        output,
        metadata,
        duration: metadata.responseTime,
      };
    } catch (error: any) {
      return this.createResult(node.id, 'failed', undefined, error.message);
    }
  }

  validate(node: WorkflowNode): boolean {
    const { config } = node.data;

    if (!config?.agentType) return false;
    if (!config?.model) return false;

    return true;
  }

  getNodeInfo() {
    return {
      type: 'aiAgent',
      name: 'AI Agent',
      description: 'AI智能体，支持对话、文本生成、向量嵌入等功能',
      inputs: [
        { name: 'message', type: 'string', required: false },
        { name: 'prompt', type: 'string', required: false },
        { name: 'data', type: 'any', required: false },
      ],
      outputs: [
        { name: 'response', type: 'string' },
        { name: 'text', type: 'string' },
        { name: 'embedding', type: 'array' },
        { name: 'model', type: 'string' },
      ],
    };
  }
}

/**
 * 条件节点执行器
 */
export class ConditionNodeExecutor extends BaseNodeExecutor {
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<NodeExecutionResult> {
    const { conditionType, config } = node.data;

    try {
      let result: any = {};

      switch (conditionType) {
        case 'if':
          result = this.evaluateIfCondition(config, context);
          break;
        case 'switch':
          result = this.evaluateSwitchCondition(config, context);
          break;
        case 'custom':
          result = this.evaluateCustomCondition(config, context);
          break;
        default:
          throw new Error(`Unsupported condition type: ${conditionType}`);
      }

      return this.createResult(node.id, 'completed', result);
    } catch (error: any) {
      return this.createResult(node.id, 'failed', undefined, error.message);
    }
  }

  private evaluateIfCondition(config: any, context: ExecutionContext): any {
    const { expression, trueValue, falseValue } = config;

    // 简化的条件评估
    let conditionResult = false;

    if (expression) {
      // 这里应该有更复杂的表达式解析器
      conditionResult = Math.random() > 0.5; // 模拟条件评估
    }

    return {
      condition: conditionResult,
      result: conditionResult ? trueValue : falseValue,
      branch: conditionResult ? 'true' : 'false',
    };
  }

  private evaluateSwitchCondition(config: any, context: ExecutionContext): any {
    const { value, cases, defaultCase } = config;

    // 简化的switch评估
    const matchedCase = cases?.find((c: any) => c.value === value) || defaultCase;

    return {
      value,
      matchedCase: matchedCase?.value || 'default',
      result: matchedCase?.result,
    };
  }

  private evaluateCustomCondition(config: any, context: ExecutionContext): any {
    const { script } = config;

    // 这里应该有安全的脚本执行环境
    return {
      script,
      result: true, // 模拟自定义条件结果
    };
  }

  validate(node: WorkflowNode): boolean {
    const { conditionType, config } = node.data;

    if (!conditionType) return false;

    if (conditionType === 'if' && !config?.expression) return false;
    if (conditionType === 'switch' && !config?.cases) return false;

    return true;
  }

  getNodeInfo() {
    return {
      type: 'condition',
      name: '条件节点',
      description: '条件判断节点，支持if、switch和自定义条件',
      inputs: [
        { name: 'value', type: 'any', required: false },
        { name: 'expression', type: 'string', required: false },
      ],
      outputs: [
        { name: 'condition', type: 'boolean' },
        { name: 'result', type: 'any' },
        { name: 'branch', type: 'string' },
      ],
    };
  }
}

/**
 * 动作节点执行器
 */
export class ActionNodeExecutor extends BaseNodeExecutor {
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<NodeExecutionResult> {
    const { actionType, config } = node.data;

    try {
      let output: any = {};

      switch (actionType) {
        case 'api':
          output = await this.executeApiAction(config);
          break;
        case 'code':
          output = await this.executeCodeAction(config);
          break;
        case 'database':
          output = await this.executeDatabaseAction(config);
          break;
        case 'email':
          output = await this.executeEmailAction(config);
          break;
        case 'file':
          output = await this.executeFileAction(config);
          break;
        default:
          throw new Error(`Unsupported action type: ${actionType}`);
      }

      return this.createResult(node.id, 'completed', output);
    } catch (error: any) {
      return this.createResult(node.id, 'failed', undefined, error.message);
    }
  }

  private async executeApiAction(config: any): Promise<any> {
    const { url, method, headers, body } = config;

    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 500 + Math.random() * 1500));

    return {
      url,
      method: method || 'GET',
      status: 200,
      response: { message: 'API call successful', data: {} },
      duration: 500 + Math.random() * 1500,
    };
  }

  private async executeCodeAction(config: any): Promise<any> {
    const { script, language } = config;

    // 模拟代码执行
    await new Promise(resolve => setTimeout(resolve, 200 + Math.random() * 800));

    return {
      script,
      language: language || 'javascript',
      output: 'Code execution result',
      exitCode: 0,
    };
  }

  private async executeDatabaseAction(config: any): Promise<any> {
    const { query, database, table } = config;

    // 模拟数据库操作
    await new Promise(resolve => setTimeout(resolve, 300 + Math.random() * 700));

    return {
      database,
      table,
      query,
      affectedRows: Math.floor(Math.random() * 100),
      duration: 300 + Math.random() * 700,
    };
  }

  private async executeEmailAction(config: any): Promise<any> {
    const { to, subject, body } = config;

    // 模拟邮件发送
    await new Promise(resolve => setTimeout(resolve, 1000 + Math.random() * 2000));

    return {
      to,
      subject,
      messageId: `msg_${Date.now()}`,
      status: 'sent',
      duration: 1000 + Math.random() * 2000,
    };
  }

  private async executeFileAction(config: any): Promise<any> {
    const { operation, path, content } = config;

    // 模拟文件操作
    await new Promise(resolve => setTimeout(resolve, 100 + Math.random() * 400));

    return {
      operation,
      path,
      size: content?.length || 0,
      status: 'completed',
      duration: 100 + Math.random() * 400,
    };
  }

  validate(node: WorkflowNode): boolean {
    const { actionType, config } = node.data;

    if (!actionType) return false;

    if (actionType === 'api' && !config?.url) return false;
    if (actionType === 'email' && !config?.to) return false;
    if (actionType === 'file' && !config?.path) return false;

    return true;
  }

  getNodeInfo() {
    return {
      type: 'action',
      name: '动作节点',
      description: '执行各种动作，包括API调用、代码执行、数据库操作等',
      inputs: [
        { name: 'config', type: 'object', required: true },
        { name: 'data', type: 'any', required: false },
      ],
      outputs: [
        { name: 'result', type: 'any' },
        { name: 'status', type: 'string' },
        { name: 'duration', type: 'number' },
      ],
    };
  }
}

/**
 * 数据处理器节点执行器
 */
export class DataProcessorNodeExecutor extends BaseNodeExecutor {
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<NodeExecutionResult> {
    const { processorType, config } = node.data;

    try {
      let output: any = {};

      switch (processorType) {
        case 'mapper':
          output = this.executeDataMapping(config, context);
          break;
        case 'filter':
          output = this.executeDataFilter(config, context);
          break;
        case 'aggregator':
          output = this.executeDataAggregator(config, context);
          break;
        case 'transformer':
          output = this.executeDataTransformer(config, context);
          break;
        default:
          throw new Error(`Unsupported processor type: ${processorType}`);
      }

      return this.createResult(node.id, 'completed', output);
    } catch (error: any) {
      return this.createResult(node.id, 'failed', undefined, error.message);
    }
  }

  private executeDataMapping(config: any, context: ExecutionContext): any {
    const { mappings, inputData } = config;

    // 简化的数据映射
    const result: any = {};

    mappings?.forEach((mapping: any) => {
      if (mapping.enabled) {
        result[mapping.targetField] = inputData?.[mapping.sourceField] || mapping.defaultValue;
      }
    });

    return {
      mappedData: result,
      mappingCount: mappings?.filter((m: any) => m.enabled).length || 0,
    };
  }

  private executeDataFilter(config: any, context: ExecutionContext): any {
    const { condition, inputData } = config;

    // 简化的数据过滤
    const filteredData = Array.isArray(inputData)
      ? inputData.filter((item: any) => Math.random() > 0.3) // 模拟过滤条件
      : inputData;

    return {
      filteredData,
      originalCount: Array.isArray(inputData) ? inputData.length : 1,
      filteredCount: Array.isArray(filteredData) ? filteredData.length : 1,
    };
  }

  private executeDataAggregator(config: any, context: ExecutionContext): any {
    const { operation, field, inputData } = config;

    // 简化的数据聚合
    let result: any = 0;

    if (Array.isArray(inputData)) {
      switch (operation) {
        case 'sum':
          result = inputData.reduce((sum: number, item: any) => sum + (Number(item[field]) || 0), 0);
          break;
        case 'avg':
          result = inputData.reduce((sum: number, item: any) => sum + (Number(item[field]) || 0), 0) / inputData.length;
          break;
        case 'count':
          result = inputData.length;
          break;
        case 'min':
          result = Math.min(...inputData.map((item: any) => Number(item[field]) || 0));
          break;
        case 'max':
          result = Math.max(...inputData.map((item: any) => Number(item[field]) || 0));
          break;
        default:
          result = inputData.length;
      }
    }

    return {
      operation,
      field,
      result,
      itemCount: Array.isArray(inputData) ? inputData.length : 0,
    };
  }

  private executeDataTransformer(config: any, context: ExecutionContext): any {
    const { transformScript, inputData } = config;

    // 简化的数据转换
    const transformedData = Array.isArray(inputData)
      ? inputData.map((item: any) => ({ ...item, transformed: true, timestamp: new Date().toISOString() }))
      : { ...inputData, transformed: true, timestamp: new Date().toISOString() };

    return {
      transformedData,
      transformScript,
      itemCount: Array.isArray(transformedData) ? transformedData.length : 1,
    };
  }

  validate(node: WorkflowNode): boolean {
    const { processorType, config } = node.data;

    if (!processorType) return false;

    if (processorType === 'mapper' && !config?.mappings) return false;
    if (processorType === 'filter' && !config?.condition) return false;

    return true;
  }

  getNodeInfo() {
    return {
      type: 'dataProcessor',
      name: '数据处理器',
      description: '数据处理节点，支持映射、过滤、聚合和转换操作',
      inputs: [
        { name: 'inputData', type: 'any', required: true },
        { name: 'config', type: 'object', required: true },
      ],
      outputs: [
        { name: 'result', type: 'any' },
        { name: 'statistics', type: 'object' },
      ],
    };
  }
}

// 导出所有执行器
export const nodeExecutors = {
  trigger: new TriggerNodeExecutor(),
  aiAgent: new AIAgentNodeExecutor(),
  condition: new ConditionNodeExecutor(),
  action: new ActionNodeExecutor(),
  dataProcessor: new DataProcessorNodeExecutor(),
};