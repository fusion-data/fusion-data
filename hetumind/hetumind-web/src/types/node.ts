export interface NodePort {
  id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array' | 'any';
  description?: string;
  required?: boolean;
}

export interface NodeStatus {
  state: 'idle' | 'running' | 'completed' | 'failed' | 'disabled';
  message?: string;
  lastRun?: string;
}

export interface BaseNodeData {
  id: string;
  type: string;
  name: string;
  description?: string;
  inputs: NodePort[];
  outputs: NodePort[];
  config: Record<string, any>;
  status: NodeStatus;
}

export interface AIAgentNodeData extends BaseNodeData {
  type: 'aiAgent';
  agentType: 'chat' | 'completion' | 'embedding' | 'image';
  model: string;
  prompt: string;
  parameters: {
    temperature?: number;
    maxTokens?: number;
    topP?: number;
    frequencyPenalty?: number;
    presencePenalty?: number;
  };
}

export interface DataProcessorNodeData extends BaseNodeData {
  type: 'dataProcessor';
  processorType: 'mapper' | 'filter' | 'aggregator' | 'transformer';
  mappingRules: MappingRule[];
  filterConditions: FilterCondition[];
}

export interface TriggerNodeData extends BaseNodeData {
  type: 'trigger';
  triggerType: 'webhook' | 'schedule' | 'manual';
  webhookConfig?: {
    url: string;
    method: 'GET' | 'POST' | 'PUT' | 'DELETE';
    headers?: Record<string, string>;
    authentication?: {
      type: 'none' | 'basic' | 'bearer';
      username?: string;
      password?: string;
      token?: string;
    };
  };
  scheduleConfig?: {
    type: 'cron' | 'interval' | 'daily';
    expression?: string;
    interval?: number;
    time?: string;
  };
}

export interface ConditionNodeData extends BaseNodeData {
  type: 'condition';
  conditionType: 'if' | 'switch';
  conditions: Condition[];
  defaultBranch?: string;
}

export interface MappingRule {
  id: string;
  sourceField: string;
  targetField: string;
  transform: 'direct' | 'function' | 'expression';
  expression?: string;
  function?: string;
}

export interface FilterCondition {
  id: string;
  field: string;
  operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'contains' | 'startsWith' | 'endsWith';
  value: any;
  logicalOperator?: 'and' | 'or';
}

export interface Condition {
  id: string;
  name: string;
  expression: string;
  description?: string;
}

export interface NodeTypeDefinition {
  type: string;
  name: string;
  description: string;
  category: string;
  icon: string;
  color: string;
  inputs: NodePort[];
  outputs: NodePort[];
  configSchema: any;
  defaultConfig: any;
}