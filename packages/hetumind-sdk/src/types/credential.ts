/**
 * Credential related types based on backend API
 */

import type { WorkflowId, OpValDateTime, OpValNumber } from './common.js';

/**
 * Credential kind enum
 */
export enum CredentialKind {
  Basic = 'basic',
  ApiKey = 'api_key',
  OAuth2 = 'oauth2',
  Database = 'database',
  Custom = 'custom',
}

/**
 * Credential entity
 */
export interface CredentialEntity {
  id: string;
  namespace_id: string;
  name: string;
  data: string; // Encrypted data
  kind: CredentialKind;
  is_managed: boolean;
  created_at: string;
  updated_at?: string;
  created_by: number;
  updated_by?: number;
  logical_deletion?: string;
}

/**
 * Credential with decrypted data
 */
export interface CredentialWithDecryptedData {
  credential: CredentialEntity;
  decrypted_data: JsonValue;
}

/**
 * Request for inserting a credential
 */
export interface CredentialForInsert {
  namespace_id: string;
  name: string;
  data: JsonValue;
  kind: CredentialKind;
  is_managed?: boolean;
  id?: string;
}

/**
 * Request for updating a credential
 */
export interface CredentialForUpdate {
  namespace_id?: string;
  name?: string;
  data?: JsonValue;
  kind?: CredentialKind;
  is_managed?: boolean;
}

/**
 * Credential filter for queries
 */
export interface CredentialFilter {
  namespace_id?: { $eq?: string };
  name?: { $eq?: string; $contains?: string };
  kind?: OpValNumber;
  is_managed?: { $eq?: boolean };
  created_at?: OpValDateTime;
  created_by?: OpValNumber;
}

/**
 * Credential query request
 */
export interface CredentialForQuery {
  options: {
    page?: number;
    limit?: number;
    offset?: number;
    sort_by?: string;
    sort_order?: 'asc' | 'desc';
  };
  filters: CredentialFilter[];
}

/**
 * Request for verifying a credential
 */
export interface VerifyCredentialRequest {
  data: JsonValue;
  kind: CredentialKind;
}

/**
 * Credential verification result
 */
export interface CredentialVerifyResult {
  is_valid: boolean;
  error?: string;
  details?: JsonValue;
}

/**
 * Credential reference information
 */
export interface CredentialReference {
  workflow_id: WorkflowId;
  workflow_name: string;
  node_name?: string;
  reference_type: string;
}

/**
 * Credential references response
 */
export interface CredentialReferencesResponse {
  credential_id: string;
  credential_name: string;
  references: CredentialReference[];
  total_references: number;
}

// Re-export JsonValue type
type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };