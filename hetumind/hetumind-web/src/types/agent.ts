export interface AIAgent {
  id: string;
  name: string;
  description?: string;
  type: 'chat' | 'completion' | 'embedding' | 'image' | 'custom';
  config: AgentConfig;
  status: 'active' | 'inactive' | 'error';
  createdAt: string;
  updatedAt: string;
  version: string;
  tags?: string[];
}

export interface AgentConfig {
  model: string;
  provider: string;
  parameters: AgentParameters;
  prompt?: string;
  systemPrompt?: string;
  tools?: AgentTool[];
  memory?: AgentMemory;
  constraints?: AgentConstraints;
}

export interface AgentParameters {
  temperature?: number;
  maxTokens?: number;
  topP?: number;
  topK?: number;
  frequencyPenalty?: number;
  presencePenalty?: number;
  stopSequences?: string[];
  responseFormat?: 'text' | 'json' | 'markdown';
}

export interface AgentTool {
  id: string;
  name: string;
  description: string;
  type: 'function' | 'api' | 'database' | 'file';
  config: any;
  enabled: boolean;
}

export interface AgentMemory {
  type: 'none' | 'short_term' | 'long_term' | 'hybrid';
  maxMessages?: number;
  vectorStore?: {
    enabled: boolean;
    collection: string;
    embeddingModel: string;
  };
}

export interface AgentConstraints {
  maxResponseTime?: number;
  allowedTopics?: string[];
  blockedTopics?: string[];
  contentFilter?: {
    enabled: boolean;
    strictness: 'low' | 'medium' | 'high';
  };
}

export interface AgentTemplate {
  id: string;
  name: string;
  description: string;
  type: string;
  category: string;
  defaultConfig: AgentConfig;
  tags?: string[];
}

export interface AgentExecution {
  id: string;
  agentId: string;
  input: any;
  output?: any;
  startTime: string;
  endTime?: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  error?: string;
  durationMs?: number;
  tokenUsage?: {
    prompt: number;
    completion: number;
    total: number;
  };
  cost?: number;
}

export interface ModelProvider {
  id: string;
  name: string;
  type: 'openai' | 'anthropic' | 'google' | 'custom';
  apiKey?: string;
  baseUrl?: string;
  models: Model[];
  enabled: boolean;
}

export interface Model {
  id: string;
  name: string;
  type: 'chat' | 'completion' | 'embedding' | 'image';
  maxTokens?: number;
  inputCost?: number;
  outputCost?: number;
  capabilities: string[];
}

export interface DataMapping {
  id: string;
  sourceField: string;
  targetField: string;
  transform: 'direct' | 'function' | 'expression';
  expression?: string;
  function?: string;
}