# Hetumind SDK

TypeScript SDK for the Hetumind AI Agent/Flow platform, based on the actual backend API endpoints.

## Installation

```bash
npm install @fusion-data/hetumind-sdk
# or
yarn add @fusion-data/hetumind-sdk
# or
pnpm add @fusion-data/hetumind-sdk
```

## Usage

```typescript
import { HetumindSDK } from '@fusion-data/hetumind-sdk';

const sdk = new HetumindSDK({
  baseURL: 'http://localhost:3000',
  token: 'your-api-token'
});

// Set authentication token
sdk.setToken('your-jwt-token');

// Workflows
const workflows = await sdk.workflows.queryWorkflows({
  options: { page: 1, limit: 10 },
  filter: { status: { eq: 100 } } // Active workflows
});

const workflow = await sdk.workflows.getWorkflow('workflow-id');
const result = await sdk.workflows.executeWorkflow('workflow-id', {
  input_data: { key: 'value' }
});

// Executions
const executions = await sdk.executions.queryExecutions({
  options: { page: 1, limit: 20 },
  filter: { workflow_id: { eq: 'workflow-id' } }
});

const execution = await sdk.executions.getExecution(result.execution_id);
const logs = await sdk.executions.getExecutionLogs(result.execution_id);
const status = await sdk.executions.getExecutionStatus(result.execution_id);

// Credentials
const credentials = await sdk.credentials.queryCredentials({
  options: { page: 1, limit: 10 },
  filters: []
});

const credential = await sdk.credentials.getCredential('credential-id');
const verification = await sdk.credentials.verifyCredential({
  data: { apiKey: 'test-key' },
  kind: 'api_key'
});

// Users
const users = await sdk.users.queryUsers({
  options: { page: 1, limit: 10 },
  filters: [{ is_active: { eq: true } }]
});

const user = await sdk.users.getUserById(123);
```

## Features

- 🚀 Full TypeScript support with complete type definitions
- 🔐 Authentication with Bearer tokens
- 📦 Workflow management (CRUD operations)
- ⚡ Workflow execution and monitoring
- 📊 Real-time execution status and logs (with SSE support)
- 🔐 Credential management with encryption support
- 👥 User management operations
- 🛡️ Built-in error handling with typed errors
- 📝 Pagination and filtering support
- 🧪 Based on actual backend API endpoints

## API Reference

### HetumindSDK

Main SDK class that provides access to all Hetumind platform features.

#### Constructor

```typescript
new HetumindSDK(config: HetumindClientConfig)
```

#### Configuration

```typescript
interface HetumindClientConfig {
  baseURL: string;           // Hetumind API base URL
  timeout?: number;          // Request timeout (default: 30000ms)
  headers?: Record<string, string>;  // Custom headers
  token?: string;            // Authentication token
}
```

#### Methods

```typescript
// Authentication
sdk.setToken(token: string): void
sdk.clearToken(): void

// Static factory method
HetumindSDK.create(config: HetumindClientConfig): HetumindSDK
```

### Workflow Operations

```typescript
// Query workflows with pagination and filtering
const workflows = await sdk.workflows.queryWorkflows({
  options: { page: 1, limit: 10 },
  filter: {
    status: { eq: 100 },  // Active workflows
    name: { like: '%test%' }
  }
});

// Get single workflow
const workflow = await sdk.workflows.getWorkflow('workflow-id');

// Create workflow
const newWorkflow = await sdk.workflows.createWorkflow({
  name: 'My Workflow',
  status: 1, // Draft
  nodes: { ... },
  connections: { ... }
});

// Update workflow
const updated = await sdk.workflows.updateWorkflow('workflow-id', {
  name: 'Updated Workflow',
  status: 100 // Active
});

// Delete workflow
await sdk.workflows.deleteWorkflow('workflow-id');

// Execute workflow
const result = await sdk.workflows.executeWorkflow('workflow-id', {
  input_data: { key: 'value' }
});

// Workflow lifecycle management
await sdk.workflows.activateWorkflow('workflow-id');
await sdk.workflows.deactivateWorkflow('workflow-id');

// Validate workflow
const validation = await sdk.workflows.validateWorkflow({
  workflow: workflowObject
});

// Duplicate workflow
const duplicate = await sdk.workflows.duplicateWorkflow('workflow-id');
```

### Execution Operations

```typescript
// Query executions with filters
const executions = await sdk.executions.queryExecutions({
  options: { page: 1, limit: 20 },
  filter: {
    workflow_id: { eq: 'workflow-id' },
    status: { eq: 100 } // Success
  }
});

// Get execution details
const execution = await sdk.executions.getExecution('execution-id');

// Get execution status (lightweight)
const status = await sdk.executions.getExecutionStatus('execution-id');

// Get execution logs
const logs = await sdk.executions.getExecutionLogs('execution-id');

// Execution control
await sdk.executions.cancelExecution('execution-id');
await sdk.executions.retryExecution('execution-id');

// Real-time execution logs streaming
const eventSource = sdk.executions.streamExecutionLogs('execution-id');
eventSource.onmessage = (event) => {
  console.log('Execution log:', JSON.parse(event.data));
};
```

### Credential Operations

```typescript
// Query credentials
const credentials = await sdk.credentials.queryCredentials({
  options: { page: 1, limit: 10 },
  filters: [
    { kind: { eq: 1 } }, // Basic auth
    { is_managed: { eq: false } }
  ]
});

// Get credential with decrypted data
const credential = await sdk.credentials.getCredential('credential-id');

// Create credential
const newCredential = await sdk.credentials.createCredential({
  namespace_id: 'namespace-123',
  name: 'My API Key',
  data: { apiKey: 'secret-key' },
  kind: 'api_key'
});

// Update credential
await sdk.credentials.updateCredential('credential-id', {
  name: 'Updated API Key',
  data: { apiKey: 'new-secret-key' }
});

// Delete credential
await sdk.credentials.deleteCredential('credential-id');

// Verify credential (without saving)
const verification = await sdk.credentials.verifyCredential({
  data: { apiKey: 'test-key' },
  kind: 'api_key'
});

// Verify stored credential
const storedVerification = await sdk.credentials.verifyStoredCredential('credential-id');

// Get credential references
const references = await sdk.credentials.getCredentialReferences('credential-id');
```

### User Operations

```typescript
// Query users
const users = await sdk.users.queryUsers({
  options: { page: 1, limit: 10 },
  filters: [
    { is_active: { eq: true } },
    { is_admin: { eq: false } }
  ]
});

// Get user by ID
const user = await sdk.users.getUserById(123);

// Update user
await sdk.users.updateUserById(123, {
  display_name: 'John Doe',
  email: 'john@example.com'
});

// Update user password
await sdk.users.updateUserPassword(123, {
  new_password: 'new-secure-password',
  old_password: 'old-password' // optional
});
```

### Error Handling

The SDK provides comprehensive error handling with typed errors:

```typescript
import { HetumindError } from '@fusion-data/hetumind-sdk';

try {
  await sdk.workflows.getWorkflow('invalid-id');
} catch (error) {
  if (error instanceof HetumindError) {
    console.error('API Error:', error.message);
    console.error('Status Code:', error.status);
    console.error('Error Code:', error.code);
    console.error('Error Details:', error.details);
  }
}
```

## Type Definitions

The SDK includes comprehensive TypeScript types for all API endpoints:

- `Workflow` - Complete workflow definition
- `WorkflowStatus` - Enum for workflow states
- `Execution` - Execution record with status and results
- `ExecutionStatus` - Enum for execution states
- `CredentialEntity` - Credential definition
- `CredentialKind` - Enum for credential types
- `UserEntity` - User profile and information
- `PageResult<T>` - Paginated response wrapper
- Filter types for complex queries

## Development

### Building

```bash
pnpm build
```

### Type Checking

```bash
pnpm type-check  # or: npx tsc --noEmit
```

### Linting

```bash
pnpm lint
```

### Formatting

```bash
pnpm format
```

## Architecture

This SDK is built based on the actual Hetumind backend API:

- **Workflows**: `/api/v1/workflows/*` endpoints
- **Executions**: `/api/v1/executions/*` endpoints
- **Credentials**: `/api/v1/credentials/*` endpoints
- **Users**: `/api/v1/users/*` endpoints

The SDK follows the same structure and naming conventions as the backend API, ensuring type safety and API compatibility.

## License

Apache-2.0 - see LICENSE file for details.