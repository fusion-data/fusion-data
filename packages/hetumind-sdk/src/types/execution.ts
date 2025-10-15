/**
 * Execution related types based on backend API
 */

import type {
  ExecutionId,
  WorkflowId,
  NodeName,
  OpValDateTime,
  OpValUuid,
  OpValNumber,
  JsonValue,
  Page,
} from './common.js';
import { ExecutionMode } from './workflow.js';

/**
 * Execution status enum
 */
export enum ExecutionStatus {
  New = 1,
  Running = 10,
  Waiting = 11,
  Retrying = 21,
  Cancelled = 97,
  Crashed = 98,
  Failed = 99,
  Success = 100,
}

/**
 * Execution definition
 */
export interface Execution {
  id: ExecutionId;
  workflow_id: WorkflowId;
  status: ExecutionStatus;
  started_at?: string;
  finished_at?: string;
  data?: JsonValue;
  error?: string;
  mode: ExecutionMode;
  triggered_by?: string;
}

/**
 * Execution data
 */
export interface ExecutionData {
  id: string;
  execution_id: ExecutionId;
  node_name: NodeName;
  data: JsonValue;
  timestamp: string;
  data_type: string;
}

/**
 * Node execution result
 */
export interface NodeExecutionResult {
  node_name: NodeName;
  status: ExecutionStatus;
  output_data?: JsonValue;
  error?: string;
  started_at: string;
  finished_at?: string;
  duration_ms?: number;
}

/**
 * Execution filter for queries
 */
export interface ExecutionFilter {
  workflow_id?: OpValUuid;
  status?: OpValNumber;
  started_at?: OpValDateTime;
  finished_at?: OpValDateTime;
  wait_till?: OpValDateTime;
}

/**
 * Execution query request
 */
export interface ExecutionForQuery {
  options: Page;
  filter: ExecutionFilter;
}

/**
 * Execution status response (lightweight)
 */
export interface ExecutionStatusResponse {
  id: ExecutionId;
  status: ExecutionStatus;
  started_at?: string;
  finished_at?: string;
  error?: string;
  progress?: number;
}

/**
 * Execution log response
 */
export type ExecutionLogResponse = ExecutionData[];

/**
 * Execution response (full details)
 */
export type ExecutionResponse = Execution;
