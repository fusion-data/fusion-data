# Hetuflow SDK WebAssembly Bindings

This document describes how to use the Hetuflow SDK WebAssembly bindings in JavaScript/TypeScript projects.

## Features

- ✅ Full WASM support with `wasm-bindgen`
- ✅ TypeScript type definitions included
- ✅ Promise-based async API
- ✅ Error handling with JavaScript Error objects
- ✅ Browser and Node.js compatibility
- ✅ Configuration management
- ✅ All Hetuflow APIs accessible

## Quick Start

### Installation

```bash
# Build the WASM package
wasm-pack build --target web --features wasm

# The package will be available in the `pkg/` directory
```

### Browser Usage

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, { WasmHetuflowClient } from './pkg/hetuflow_sdk.js';

        async function main() {
            // Initialize the WASM module
            await init();

            // Create a client
            const client = new WasmHetuflowClient("http://localhost:8080");

            // Use the client
            try {
                const agents = await client.agents().query({ page: 1, limit: 10 });
                console.log("Agents:", agents);
            } catch (error) {
                console.error("Error:", error);
            }
        }

        main();
    </script>
</head>
</html>
```

### Node.js Usage

```javascript
const { WasmHetuflowClient } = require('./pkg/hetuflow_sdk.js');

async function main() {
    // Note: In Node.js, you might need additional setup for WASM support
    const client = new WasmHetuflowClient("http://localhost:8080");

    const agents = await client.agents().query({});
    console.log("Agents:", agents);
}

main().catch(console.error);
```

### TypeScript Usage

```typescript
import { WasmHetuflowClient, WasmConfig, Agent, QueryParams } from './pkg/hetuflow_sdk.js';

class HetuflowService {
    private client: WasmHetuflowClient;

    constructor(baseURL: string, authToken?: string) {
        const config = new WasmConfig(baseURL);
        if (authToken) {
            config.set_auth_token(authToken);
        }
        this.client = WasmHetuflowClient.withConfig(config);
    }

    async getAgents(params?: QueryParams): Promise<{ items: Agent[], total: number }> {
        return await this.client.agents().query(params || {});
    }

    async createAgent(data: { name: string, capabilities: string[] }): Promise<Agent> {
        return await this.client.agents().create(data);
    }
}

// Usage
const service = new HetuflowService("http://localhost:8080", "your-token");
const agents = await service.getAgents({ page: 1, limit: 20 });
```

## API Reference

### WasmHetuflowClient

The main client class for interacting with Hetuflow.

#### Constructor

```typescript
constructor(base_url: string)
```

Creates a new client with the given base URL.

#### Static Methods

```typescript
static withConfig(config: WasmConfig): WasmHetuflowClient
```

Creates a new client with custom configuration.

#### API Accessors

```typescript
get agents(): WasmAgentsApi
get jobs(): WasmJobsApi
get tasks(): WasmTasksApi
get schedules(): WasmSchedulesApi
get task_instances(): WasmTaskInstancesApi
get servers(): WasmServersApi
get system(): WasmSystemApi
get gateway(): WasmGatewayApi
get auth(): WasmAuthApi
```

### WasmConfig

Configuration class for the Hetuflow client.

#### Constructor

```typescript
constructor(base_url: string)
```

#### Methods

```typescript
set_auth_token(token: string | null): void
get auth_token(): string | null

set_header(name: string, value: string): void

set timeout(milliseconds: number): void
get timeout(): number

set compression(enabled: boolean): void
get compression(): boolean
```

### API Classes

All API classes (WasmAgentsApi, WasmJobsApi, etc.) implement the same basic interface:

```typescript
interface BaseApi {
    query(params?: QueryParams): Promise<QueryResult<T>>
    get(id: string): Promise<T>
    create(data: any): Promise<T>
    update(id: string, data: any): Promise<T>
    delete(id: string): Promise<void>
}
```

### QueryParams

```typescript
interface QueryParams {
    page?: number;
    limit?: number;
    sort?: string;
    order?: 'asc' | 'desc';
    [key: string]: any;
}
```

### QueryResult

```typescript
interface QueryResult<T = any> {
    items: T[];
    total: number;
    page: number;
    limit: number;
    pages: number;
}
```

## Error Handling

All API methods throw JavaScript Error objects when something goes wrong:

```typescript
try {
    const agents = await client.agents().query({});
} catch (error) {
    console.error("API Error:", error.message);
    // Error types include:
    // - Network errors
    // - Validation errors
    // - Configuration errors
    // - API errors (from the server)
}
```

## Utility Functions

### WasmUtils

Utility functions for JavaScript interop:

```typescript
// Convert JavaScript Error to string
WasmUtils.errorToString(error: Error): string

// Check if a value is a Promise
WasmUtils.isPromise(value: any): boolean

// Get current timestamp in milliseconds
WasmUtils.timestampMs(): number

// Safe JSON stringification
WasmUtils.safeJsonStringify(value: any): string
```

### WasmError

Error creation utilities:

```typescript
// Create error from SDK error message
WasmError.from_sdk_error(message: string): Error

// Create network error
WasmError.network_error(message: string): Error

// Create validation error
WasmError.validation_error(message: string): Error

// Create configuration error
WasmError.config_error(message: string): Error
```

## Examples

### Basic Usage

```javascript
import init, { WasmHetuflowClient } from './pkg/hetuflow_sdk.js';

// Initialize WASM
await init();

// Create client
const client = new WasmHetuflowClient("http://localhost:8080");

// List agents
const agents = await client.agents().query({ limit: 10 });
console.log("Found agents:", agents.items);

// Create a job
const job = await client.jobs().create({
    name: "Test Job",
    description: "A test job created from WASM",
    tasks: [
        {
            name: "Task 1",
            type: "shell",
            input: { command: "echo 'Hello from WASM!'" }
        }
    ]
});
console.log("Created job:", job);
```

### With Authentication

```javascript
import { WasmHetuflowClient, WasmConfig } from './pkg/hetuflow_sdk.js';

const config = new WasmConfig("http://localhost:8080");
config.set_auth_token("your-jwt-token-here");
config.set_timeout(60000); // 60 seconds

const client = WasmHetuflowClient.withConfig(config);
```

### Error Handling

```javascript
async function safeApiCall() {
    try {
        const result = await client.agents().query({});
        return result;
    } catch (error) {
        if (error.message.includes("Network")) {
            console.log("Network error - check your connection");
        } else if (error.message.includes("Authentication")) {
            console.log("Auth error - check your token");
        } else {
            console.log("Other error:", error.message);
        }
        throw error;
    }
}
```

## Building

To build the WASM package:

```bash
# Development build
wasm-pack build --target web --features wasm

# Production build (optimized)
wasm-pack build --target web --features wasm --release

# Build for different targets
wasm-pack build --target nodejs --features wasm    # Node.js
wasm-pack build --target bundler --features wasm   # Webpack/other bundlers
wasm-pack build --target web --features wasm       # Direct browser use
```

## Limitations

1. **Placeholder Implementation**: The current API methods return placeholder errors. Full implementation requires connecting to the actual Rust API methods.

2. **Browser Compatibility**: Requires modern browsers with WebAssembly support.

3. **Node.js**: May require additional setup for WASM support in older Node.js versions.

4. **File Size**: WASM files can be large; consider code splitting for production.

## Development

To extend the WASM bindings:

1. Add new functions to `src/wasm.rs`
2. Update TypeScript definitions in `hetuflow_sdk.d.ts`
3. Rebuild with `wasm-pack build`
4. Test with the provided HTML example

## Next Steps

1. **Full API Implementation**: Connect the placeholder methods to actual Rust API implementations
2. **Streaming Support**: Add support for streaming responses
3. **Authentication Helpers**: Add helper methods for different auth flows
4. **Type Generation**: Auto-generate TypeScript types from Rust models
5. **Performance Optimization**: Optimize WASM bundle size and performance