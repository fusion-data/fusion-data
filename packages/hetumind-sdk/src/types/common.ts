/**
 * Common types and interfaces
 */

// Re-export page and operation types from fusionsql
export type {
  Page,
  PageResult,
  Paged,
  OrderDirection,
  OpValString,
  OpValBool,
  OpValNumber,
  OpValDateTime,
  OpValUuid,
} from '@fusion-data/fusionsql';

// Custom ID types (as used in the backend)
export type WorkflowId = string;
export type ExecutionId = string;
export type NodeName = string;
export type ConnectionKind = string;
export type UserId = string;

// Additional types needed for backend API
export interface IdUuidResult {
  id: string;
}

// JSON value type
export type JsonValue =
  | string
  | number
  | boolean
  | null
  | JsonValue[]
  | { [key: string]: JsonValue };

// Field mask for updates
export interface FieldMask {
  paths: string[];
}
