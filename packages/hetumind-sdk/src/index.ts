/**
 * Hetumind SDK - TypeScript SDK for the Hetumind AI Agent/Flow platform
 *
 * This SDK provides a TypeScript interface for interacting with the Hetumind platform,
 * including workflow management, execution monitoring, credential management, and user management.
 *
 * @version 2.0.0
 *
 * @example
 * ```typescript
 * import { HetumindSDK } from '@fusion-data/hetumind-sdk';
 *
 * const sdk = new HetumindSDK({
 *   baseURL: 'http://localhost:3000',
 *   token: 'your-api-token'
 * });
 *
 * // Workflows
 * const workflows = await sdk.workflows.queryWorkflows({
 *   options: { page: 1, limit: 10 },
 *   filter: { status: { eq: 100 } } // Active workflows
 * });
 *
 * const workflow = await sdk.workflows.getWorkflow('workflow-id');
 *
 * const result = await sdk.workflows.executeWorkflow('workflow-id', {
 *   input_data: { key: 'value' }
 * });
 *
 * // Executions
 * const executions = await sdk.executions.queryExecutions({
 *   options: { page: 1, limit: 20 },
 *   filter: { workflow_id: { eq: 'workflow-id' } }
 * });
 *
 * const execution = await sdk.executions.getExecution(result.execution_id);
 * const logs = await sdk.executions.getExecutionLogs(result.execution_id);
 *
 * // Credentials
 * const credentials = await sdk.credentials.queryCredentials({
 *   options: { page: 1, limit: 10 },
 *   filters: []
 * });
 *
 * const credential = await sdk.credentials.getCredential('credential-id');
 *
 * // Users
 * const users = await sdk.users.queryUsers({
 *   options: { page: 1, limit: 10 },
 *   filters: [{ is_active: { eq: true } }]
 * });
 * ```
 */

// Main SDK class
export { HetumindSDK, default as default } from './sdk.js';

// Export client related
export { HetumindClient, HetumindError, type HetumindClientConfig } from './utils/client.js';

// Export all API classes
export { WorkflowAPI } from './api/workflows.js';
export { ExecutionAPI } from './api/executions.js';
export { CredentialAPI } from './api/credentials.js';
export { UserAPI } from './api/users.js';

// Export all type definitions
export * from './types/index.js';