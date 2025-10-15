/**
 * Workflow related types based on backend API
 */

import type {
  WorkflowId,
  NodeName,
  ConnectionKind,
  OpValString,
  OpValUuid,
  OpValBool,
  OpValNumber,
  JsonValue,
  FieldMask,
  Page,
} from './common.js';

/**
 * Workflow status enum
 */
export enum WorkflowStatus {
  Draft = 1,
  Disabled = 99,
  Active = 100,
}

/**
 * Execution mode enum
 */
export enum ExecutionMode {
  Local = 1,
  Distributed = 2,
}

/**
 * Error handling strategy
 */
export enum ErrorHandlingStrategy {
  StopOnFirstError = 'StopOnFirstError',
  ContinueOnError = 'ContinueOnError',
  ErrorNode = 'ErrorNode',
}

/**
 * Workflow settings
 */
export interface WorkflowSettings {
  execution_timeout?: number;
  error_handling?: ErrorHandlingStrategy;
  execution_mode?: ExecutionMode;
  remark?: string;
}

/**
 * Workflow metadata
 */
export interface WorkflowMeta {
  credentials_setup_completed?: boolean;
  template_id?: string;
}

/**
 * Connection definition
 */
export interface Connection {
  node_name: NodeName;
  connection_kind: ConnectionKind;
  target_index?: number;
}

/**
 * Workflow node definition
 */
export interface WorkflowNode {
  id: string;
  name: NodeName;
  kind: string;
  display_name?: string;
  parameters?: JsonValue;
  position?: {
    x: number;
    y: number;
  };
  webhook_id?: string;
  credentials?: Record<string, any>;
  always_output_data?: boolean;
  execute_mode?: 'EachItem' | 'All';
  notes_in_flow?: boolean;
  notes?: string;
  on_error?: 'Stop' | 'Continue' | 'ErrorOutput';
  max_tries?: number;
  wait_between_tries?: number;
  timeout?: number;
}

/**
 * Complete workflow definition
 */
export interface Workflow {
  id: WorkflowId;
  name: string;
  status: WorkflowStatus;
  version?: WorkflowId;
  settings: WorkflowSettings;
  meta: WorkflowMeta;
  nodes: WorkflowNode[];
  connections: Record<NodeName, Record<ConnectionKind, Connection[]>>;
  pin_data: Record<string, JsonValue[]>;
  static_data?: JsonValue;
}

/**
 * Request for creating a workflow
 */
export interface WorkflowForCreate {
  id?: WorkflowId;
  name: string;
  status?: WorkflowStatus;
  nodes?: JsonValue;
  connections?: JsonValue;
  settings?: JsonValue;
  static_data?: JsonValue;
  pin_data?: JsonValue;
  version_id?: WorkflowId;
  meta?: JsonValue;
}

/**
 * Request for updating a workflow
 */
export interface WorkflowForUpdate {
  name?: string;
  status?: WorkflowStatus;
  nodes?: JsonValue;
  connections?: JsonValue;
  settings?: JsonValue;
  static_data?: JsonValue;
  pin_data?: JsonValue;
  version_id?: WorkflowId;
  trigger_count?: number;
  meta?: JsonValue;
  parent_folder_id?: string;
  is_archived?: boolean;
  field_mask?: FieldMask;
}

/**
 * Workflow filter for queries
 */
export interface WorkflowFilter {
  name?: OpValString;
  status?: OpValNumber;
  version_id?: OpValUuid;
  trigger_count?: OpValNumber;
  parent_folder_id?: OpValUuid;
  is_archived?: OpValBool;
}

/**
 * Workflow query request
 */
export interface WorkflowForQuery {
  options: Page;
  filter: WorkflowFilter;
}

/**
 * Request for validating a workflow
 */
export interface ValidateWorkflowRequest {
  id?: WorkflowId;
  workflow?: Workflow;
}

/**
 * Response for workflow validation
 */
export interface ValidateWorkflowResponse {
  is_valid: boolean;
  errors?: ValidationError[];
}

/**
 * Validation error type
 */
export interface ValidationError {
  type: string;
  message: string;
  details?: JsonValue;
}

/**
 * Request for executing a workflow
 */
export interface ExecuteWorkflowRequest {
  input_data?: Record<string, JsonValue>;
}

/**
 * Response with execution ID
 */
export interface ExecutionIdResponse {
  execution_id: string;
}
